// Copyright (c) 2018-2021 The MobileCoin Foundation

//! The TestClient supports two use-cases:
//! - End-to-end testing (used in continuous deployment)
//! - Fog canary (sends transactions in prod to alert if it fails, and collect
//!   timings)

use crate::{counters, error::TestClientError};

use hex_fmt::HexList;
use mc_account_keys::ShortAddressHash;
use mc_common::logger::{log, Logger};
use mc_crypto_rand::McRng;
use mc_fog_sample_paykit::{AccountKey, Client, ClientBuilder, TransactionStatus, Tx};
use mc_fog_uri::{FogLedgerUri, FogViewUri};
use mc_sgx_css::Signature;
use mc_transaction_core::{constants::RING_SIZE, tokens::Mob, BlockIndex, Token};
use mc_transaction_std::MemoType;
use mc_util_telemetry::{
    block_span_builder, mark_span_as_active, telemetry_static_key, tracer, Context, Key, Span,
    SpanKind, Tracer,
};
use mc_util_uri::ConsensusClientUri;
use more_asserts::assert_gt;
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

/// Telemetry: block index currently being worked on
const TELEMETRY_BLOCK_INDEX_KEY: Key = telemetry_static_key!("block-index");

/// Policy for different kinds of timeouts.
/// In acceptance testing, we want to fail fast if things take too long.
/// When measuring prod, we usually don't.
#[derive(Debug, Clone, Serialize)]
pub struct TestClientPolicy {
    /// Whether to fail fast if a deadline is passed. In the test client, we
    /// want this, and in the canary, we don't, because we want to continue
    /// measuring the time it takes.
    pub fail_fast_on_deadline: bool,
    /// An amount of time to wait for a submitted Tx to land before returning an
    /// error
    pub tx_submit_deadline: Duration,
    /// An amount of time to wait for a submitted Tx to be recieved before
    /// returning an error
    pub tx_receive_deadline: Duration,
    /// An amount of time to wait before running the double spend test
    pub double_spend_wait: Duration,
    /// An amount of time to backoff before polling again, when polling fog
    /// servers
    pub polling_wait: Duration,
    /// A transaction amount to send
    pub transfer_amount: u64,
    /// Whether to test RTH memos
    pub test_rth_memos: bool,
}

impl Default for TestClientPolicy {
    fn default() -> Self {
        Self {
            fail_fast_on_deadline: false,
            tx_submit_deadline: Duration::from_secs(10),
            tx_receive_deadline: Duration::from_secs(10),
            double_spend_wait: Duration::from_secs(10),
            polling_wait: Duration::from_millis(50),
            transfer_amount: Mob::MINIMUM_FEE,
            test_rth_memos: false,
        }
    }
}

/// An object which can run test transfers
pub struct TestClient {
    policy: TestClientPolicy,
    account_keys: Vec<AccountKey>,
    consensus_uris: Vec<ConsensusClientUri>,
    fog_ledger: FogLedgerUri,
    fog_view: FogViewUri,
    consensus_sig: Option<Signature>,
    fog_ingest_sig: Option<Signature>,
    fog_ledger_sig: Option<Signature>,
    fog_view_sig: Option<Signature>,
    tx_info: Arc<TxInfo>,
    health_tracker: Arc<HealthTracker>,
    logger: Logger,
}

impl TestClient {
    /// Create a new test client object
    ///
    /// Arguments:
    /// * policy: The test client policy, which includes a bunch of timing
    ///   configurations
    /// * account_keys: The account private keys to use for the test. Should be
    ///   at least two.
    /// * consensus_uris: The various consensus uris to hit as part of the test.
    /// * fog_ledger: The uri for the fog ledger service
    /// * fog_view: The uri for the fog view service
    /// * logger: Logger to use
    pub fn new(
        policy: TestClientPolicy,
        account_keys: Vec<AccountKey>,
        consensus_uris: Vec<ConsensusClientUri>,
        fog_ledger: FogLedgerUri,
        fog_view: FogViewUri,
        logger: Logger,
    ) -> Self {
        let tx_info = Arc::new(Default::default());
        // The test client uses accounts_keys.len() many clients in round robin
        // fashion. If we set the healing time of health tracker to be this number,
        // then we will heal when every client has successfully transferred again.
        let health_tracker = Arc::new(HealthTracker::new(account_keys.len()));
        Self {
            policy,
            account_keys,
            consensus_uris,
            fog_ledger,
            fog_view,
            logger,
            consensus_sig: None,
            fog_ingest_sig: None,
            fog_ledger_sig: None,
            fog_view_sig: None,
            tx_info,
            health_tracker,
        }
    }

    /// Set the consensus sigstruct used by the clients
    pub fn consensus_sigstruct(self, sig: Option<Signature>) -> Self {
        let mut retval = self;
        retval.consensus_sig = sig;
        retval
    }

    /// Set the fog ingest sigstruct used by the clients
    pub fn fog_ingest_sigstruct(self, sig: Option<Signature>) -> Self {
        let mut retval = self;
        retval.fog_ingest_sig = sig;
        retval
    }

    /// Set the fog ledger sigstruct used by the clients
    pub fn fog_ledger_sigstruct(self, sig: Option<Signature>) -> Self {
        let mut retval = self;
        retval.fog_ledger_sig = sig;
        retval
    }

    /// Set the fog view sigstruct used by the clients
    pub fn fog_view_sigstruct(self, sig: Option<Signature>) -> Self {
        let mut retval = self;
        retval.fog_view_sig = sig;
        retval
    }

    /// Build the clients
    ///
    /// Arguments:
    /// * client count: the number of clients to build. Need at least two for
    ///   the test to work
    fn build_clients(&self, client_count: usize) -> Vec<Arc<Mutex<Client>>> {
        let mut clients = Vec::new();
        // Need at least 2 clients to send transactions to each other.
        assert_gt!(client_count, 1);

        // Build an address book for each client (for memos)
        let address_book: Vec<_> = self
            .account_keys
            .iter()
            .map(|x| x.default_subaddress())
            .collect();

        for (i, account_key) in self.account_keys.iter().enumerate() {
            log::debug!(
                self.logger,
                "Now building client for account_key {} {:?}",
                i,
                account_key
            );
            let uri = &self.consensus_uris[i % self.consensus_uris.len()];
            let client = ClientBuilder::new(
                uri.clone(),
                self.fog_view.clone(),
                self.fog_ledger.clone(),
                account_key.clone(),
                self.logger.clone(),
            )
            .ring_size(RING_SIZE)
            .use_rth_memos(self.policy.test_rth_memos)
            .address_book(address_book.clone())
            .consensus_sig(self.consensus_sig.clone())
            .fog_ingest_sig(self.fog_ingest_sig.clone())
            .fog_ledger_sig(self.fog_ledger_sig.clone())
            .fog_view_sig(self.fog_view_sig.clone())
            .build();
            clients.push(Arc::new(Mutex::new(client)));
        }
        clients
    }

    /// Conduct a transfer between two clients, according to the policy
    /// Returns the transaction and the block count of the node it was submitted
    /// to.
    ///
    /// This only builds and submits the transaction, it does not confirm it
    fn transfer(
        &self,
        source_client: &mut Client,
        target_client: &mut Client,
    ) -> Result<(Tx, u64), TestClientError> {
        self.tx_info.clear();
        let target_address = target_client.get_account_key().default_subaddress();
        log::debug!(
            self.logger,
            "Attempting to transfer {} ({})",
            self.policy.transfer_amount,
            source_client.consensus_service_address()
        );

        // First do a balance check to flush out any spent txos
        let tracer = tracer!();
        tracer.in_span("pre_transfer_balance_check", |_cx| {
            source_client
                .check_balance()
                .map_err(TestClientError::CheckBalance)
        })?;

        let mut rng = McRng::default();
        assert!(target_address.fog_report_url().is_some());

        // Get the current fee from consensus
        let fee = source_client.get_fee().unwrap_or(Mob::MINIMUM_FEE);

        // Scope for build operation
        let transaction = {
            let start = Instant::now();
            let transaction = source_client
                .build_transaction(self.policy.transfer_amount, &target_address, &mut rng, fee)
                .map_err(TestClientError::BuildTx)?;
            counters::TX_BUILD_TIME.observe(start.elapsed().as_secs_f64());
            transaction
        };
        self.tx_info.set_tx(&transaction);

        // Scope for send operation
        let block_count = {
            let start = Instant::now();
            let block_count = source_client
                .send_transaction(&transaction)
                .map_err(TestClientError::SubmitTx)?;
            counters::TX_SEND_TIME.observe(start.elapsed().as_secs_f64());
            block_count
        };
        self.tx_info.set_tx_propose_block_count(block_count);
        Ok((transaction, block_count))
    }

    /// Waits for a transaction to be accepted by the network
    ///
    /// Uses the client to poll a fog service until the submitted transaction
    /// either appears or has expired. Panics if the transaction is not
    /// accepted.
    ///
    /// Arguments:
    /// * client: The client to use for this check
    /// * transaction: The (submitted) transaction to check if it landed
    ///
    /// Returns:
    /// * A block index in which the transaction landed, or a test client error.
    fn ensure_transaction_is_accepted(
        &self,
        client: &mut Client,
        transaction: &Tx,
    ) -> Result<BlockIndex, TestClientError> {
        let tracer = tracer!();
        tracer.in_span("ensure_transaction_is_accepted", |_cx| {
            // Wait until ledger server can see all of these key images
            let mut deadline = Some(Instant::now() + self.policy.tx_submit_deadline);
            loop {
                match client
                    .is_transaction_present(transaction)
                    .map_err(TestClientError::ConfirmTx)?
                {
                    TransactionStatus::Appeared(block_index) => return Ok(block_index),
                    TransactionStatus::Expired => return Err(TestClientError::TxExpired),
                    TransactionStatus::Unknown => {}
                }
                deadline = if let Some(deadline) = deadline {
                    if Instant::now() > deadline {
                        counters::TX_CONFIRMED_DEADLINE_EXCEEDED_COUNT.inc();
                        // Announce unhealthy status once the deadline is exceeded, even if we don't
                        // fail fast
                        self.health_tracker.announce_failure();
                        log::error!(
                            self.logger,
                            "TX appear deadline ({:?}) was exceeded: {}",
                            self.policy.tx_receive_deadline,
                            self.tx_info
                        );
                        if self.policy.fail_fast_on_deadline {
                            return Err(TestClientError::SubmittedTxTimeout);
                        }
                        None
                    } else {
                        Some(deadline)
                    }
                } else {
                    None
                };
                log::info!(
                    self.logger,
                    "Checking transaction again after {:?}...",
                    self.policy.polling_wait
                );
                std::thread::sleep(self.policy.polling_wait);
            }
        })
    }

    /// Ensure that after all fog servers have caught up and the client has data
    /// up to a certain number of blocks, the client computes the expected
    /// balance.
    ///
    /// Arguments:
    /// * block_index: The block_index containing new transactions that must be
    ///   in the balance
    /// * expected_balance: The expected balance to compute after this
    ///   block_index is included
    fn ensure_expected_balance_after_block(
        &self,
        client: &mut Client,
        block_index: BlockIndex,
        expected_balance: u64,
    ) -> Result<(), TestClientError> {
        let start = Instant::now();
        let mut deadline = Some(start + self.policy.tx_receive_deadline);

        loop {
            let (new_balance, new_block_count) = client
                .check_balance()
                .map_err(TestClientError::CheckBalance)?;

            // Wait for client cursor to include the index where the transaction landed.
            if u64::from(new_block_count) > block_index {
                log::debug!(
                    self.logger,
                    "Txo cursor now {} > block_index {}, after {:?}",
                    new_block_count,
                    block_index,
                    start.elapsed()
                );
                log::debug!(
                    self.logger,
                    "Expected balance: {:?}, and got: {:?}",
                    expected_balance,
                    new_balance
                );
                if expected_balance != new_balance {
                    return Err(TestClientError::BadBalance(expected_balance, new_balance));
                }
                log::info!(self.logger, "Successful transfer");
                return Ok(());
            }
            deadline = if let Some(deadline) = deadline {
                if Instant::now() > deadline {
                    counters::TX_RECEIVED_DEADLINE_EXCEEDED_COUNT.inc();
                    // Announce unhealthy status once the deadline is exceeded, even if we don't
                    // fail fast
                    self.health_tracker.announce_failure();
                    log::error!(
                        self.logger,
                        "TX receive deadline ({:?}) was exceeded: {}",
                        self.policy.tx_receive_deadline,
                        self.tx_info
                    );
                    if self.policy.fail_fast_on_deadline {
                        return Err(TestClientError::TxTimeout);
                    }
                    None
                } else {
                    Some(deadline)
                }
            } else {
                None
            };

            log::trace!(
                self.logger,
                "num_blocks = {} but tx expected in block index = {}, retry in {:?}...",
                new_block_count,
                block_index,
                self.policy.polling_wait
            );
            std::thread::sleep(self.policy.polling_wait);
        }
    }

    /// Attempt a double spend on the given transaction.
    fn attempt_double_spend(
        &self,
        client: &mut Client,
        transaction: &Tx,
    ) -> Result<(), TestClientError> {
        log::info!(self.logger, "Now attempting spent key image test");
        // NOTE: without the wait, the call to send_transaction would succeed.
        //       This test is a little ambiguous because it is testing that
        //       the transaction cannot even be sent, not just that it fails to
        //       pass consensus.
        std::thread::sleep(self.policy.double_spend_wait);
        match client.send_transaction(transaction) {
            Ok(_) => {
                log::error!(
                    self.logger,
                    "Double spend succeeded. Check whether the ledger is up-to-date"
                );
                Err(TestClientError::DoubleSpend)
            }
            Err(e) => {
                log::info!(self.logger, "Double spend failed with {:?}", e);
                Ok(())
            }
        }
    }

    /// Conduct a test transfer from source client to target client
    ///
    /// Arguments:
    /// * source_client: The client to send from
    /// * source_client_index: The index of this client in the list of clients
    ///   (for debugging info)
    /// * target_client: The client to receive the Tx
    /// * target_client_index: The index of this client in the list of clients
    ///   (for debugging info)
    fn test_transfer(
        &self,
        source_client: Arc<Mutex<Client>>,
        source_client_index: usize,
        target_client: Arc<Mutex<Client>>,
        target_client_index: usize,
    ) -> Result<Tx, TestClientError> {
        self.tx_info.clear();
        let tracer = tracer!();

        let mut source_client_lk = source_client.lock().expect("mutex poisoned");
        let mut target_client_lk = target_client.lock().expect("mutex poisoned");
        let src_address_hash =
            ShortAddressHash::from(&source_client_lk.get_account_key().default_subaddress());
        let tgt_address_hash =
            ShortAddressHash::from(&target_client_lk.get_account_key().default_subaddress());

        let (src_balance, tgt_balance) = tracer.in_span(
            "test_transfer_pre_checks",
            |_cx| -> Result<(u64, u64), TestClientError> {
                let (src_balance, src_cursor) = source_client_lk
                    .check_balance()
                    .map_err(TestClientError::CheckBalance)?;
                log::info!(
                    self.logger,
                    "client {} has a balance of {} after {} blocks",
                    source_client_index,
                    src_balance,
                    src_cursor
                );
                let (tgt_balance, tgt_cursor) = target_client_lk
                    .check_balance()
                    .map_err(TestClientError::CheckBalance)?;
                log::info!(
                    self.logger,
                    "client {} has a balance of {} after {} blocks",
                    target_client_index,
                    tgt_balance,
                    tgt_cursor
                );
                if src_balance == 0 || tgt_balance == 0 {
                    return Err(TestClientError::ZeroBalance);
                }

                Ok((src_balance, tgt_balance))
            },
        )?;

        let fee = source_client_lk.get_fee().unwrap_or(Mob::MINIMUM_FEE);
        let transfer_start = std::time::SystemTime::now();
        let (transaction, block_count) =
            self.transfer(&mut source_client_lk, &mut target_client_lk)?;

        let mut span = block_span_builder(&tracer, "test_iteration", block_count)
            .with_start_time(transfer_start)
            .start(&tracer);
        span.set_attribute(TELEMETRY_BLOCK_INDEX_KEY.i64(block_count as i64));
        let _active = mark_span_as_active(span);

        let start = Instant::now();

        drop(target_client_lk);
        let mut receive_tx_worker = ReceiveTxWorker::new(
            target_client,
            tgt_balance,
            tgt_balance + self.policy.transfer_amount,
            self.policy.clone(),
            Some(src_address_hash),
            self.tx_info.clone(),
            self.health_tracker.clone(),
            self.logger.clone(),
            Context::current(),
        );

        // Wait for key images to land in ledger server
        let transaction_appeared =
            self.ensure_transaction_is_accepted(&mut source_client_lk, &transaction)?;

        counters::TX_CONFIRMED_TIME.observe(start.elapsed().as_secs_f64());

        // Tell the receive tx worker in what block the transaction appeared
        receive_tx_worker.relay_tx_appeared(transaction_appeared);

        // Wait for tx to land in fog view server
        // This test will be as flakey as the accessibility/fees of consensus
        log::info!(self.logger, "Checking balance for source");
        tracer.in_span("ensure_expected_balance_after_block", |_cx| {
            self.ensure_expected_balance_after_block(
                &mut source_client_lk,
                transaction_appeared,
                src_balance - self.policy.transfer_amount - fee,
            )
        })?;

        // Wait for receive tx worker to successfully get the transaction
        receive_tx_worker.join()?;

        if self.policy.test_rth_memos {
            // Ensure source client got a destination memo, as expected for recoverable
            // transcation history
            match source_client_lk.get_last_memo() {
                Ok(Some(memo)) => match memo {
                    MemoType::Destination(memo) => {
                        if memo.get_total_outlay() != self.policy.transfer_amount + fee {
                            log::error!(self.logger, "Destination memo had wrong total outlay, found {}, expected {}. Tx Info: {}", memo.get_total_outlay(), self.policy.transfer_amount + fee, self.tx_info);
                            return Err(TestClientError::UnexpectedMemo);
                        }
                        if memo.get_fee() != fee {
                            log::error!(
                                    self.logger,
                                    "Destination memo had wrong fee, found {}, expected {}. Tx Info: {}",
                                    memo.get_fee(),
                                    fee,
                                    self.tx_info
                                );
                            return Err(TestClientError::UnexpectedMemo);
                        }
                        if memo.get_num_recipients() != 1 {
                            log::error!(self.logger, "Destination memo had wrong num_recipients, found {}, expected 1. TxInfo: {}", memo.get_num_recipients(), self.tx_info);
                            return Err(TestClientError::UnexpectedMemo);
                        }
                        if *memo.get_address_hash() != tgt_address_hash {
                            log::error!(self.logger, "Destination memo had wrong address hash, found {:?}, expected {:?}. Tx Info: {}", memo.get_address_hash(), tgt_address_hash, self.tx_info);
                            return Err(TestClientError::UnexpectedMemo);
                        }
                    }
                    _ => {
                        log::error!(
                            self.logger,
                            "Source Client: Unexpected memo type. Tx Info: {}",
                            self.tx_info
                        );
                        return Err(TestClientError::UnexpectedMemo);
                    }
                },
                Ok(None) => {
                    log::error!(
                        self.logger,
                        "Source Client: Missing memo. Tx Info: {}",
                        self.tx_info
                    );
                    return Err(TestClientError::UnexpectedMemo);
                }
                Err(err) => {
                    log::error!(
                        self.logger,
                        "Source Client: Memo parse error: {}. TxInfo: {}",
                        err,
                        self.tx_info
                    );
                    return Err(TestClientError::InvalidMemo);
                }
            }
        }
        Ok(transaction)
    }

    /// Run a test that lasts a fixed duration and fails fast on an error
    ///
    /// Arguments:
    /// * num_transactions: The number of transactions to run
    pub fn run_test(&self, num_transactions: usize) -> Result<(), TestClientError> {
        let client_count = self.account_keys.len() as usize;
        assert!(client_count > 1);
        log::debug!(self.logger, "Creating {} clients", client_count);
        let clients = self.build_clients(client_count);

        log::debug!(self.logger, "Generating and testing transactions");

        let start_time = Instant::now();
        for ti in 0..num_transactions {
            log::debug!(self.logger, "Transation: {:?}", ti);

            let source_index = ti % client_count;
            let target_index = (ti + 1) % client_count;
            let source_client = clients[source_index].clone();
            let target_client = clients[target_index].clone();

            let transaction = self.test_transfer(
                source_client.clone(),
                source_index,
                target_client,
                target_index,
            )?;

            // Attempt double spend on the last transaction. This is an expensive test.
            if ti == num_transactions - 1 {
                let mut source_client_lk = source_client.lock().expect("mutex poisoned");
                self.attempt_double_spend(&mut source_client_lk, &transaction)?;
            }
        }
        log::debug!(
            self.logger,
            "{} transactions took {}s",
            num_transactions,
            start_time.elapsed().as_secs()
        );
        Ok(())
    }

    /// Run test transactions continuously, handling errors by incrementing
    /// prometheus counters
    ///
    /// Arguments:
    /// * period: The amount of time we wait between test transfers
    pub fn run_continuously(&self, period: Duration) {
        let client_count = self.account_keys.len() as usize;
        assert!(client_count > 1);
        log::debug!(self.logger, "Creating {} clients", client_count);
        let clients = self.build_clients(client_count);

        log::debug!(self.logger, "Generating and testing transactions");

        let mut ti = 0usize;
        loop {
            log::debug!(self.logger, "Transaction: {:?}", ti);

            let source_index = ti % client_count;
            let target_index = (ti + 1) % client_count;
            let source_client = clients[source_index].clone();
            let target_client = clients[target_index].clone();

            match self.test_transfer(source_client, source_index, target_client, target_index) {
                Ok(_) => {
                    log::info!(self.logger, "Transfer succeeded");
                    counters::TX_SUCCESS_COUNT.inc();
                }
                Err(err) => {
                    log::error!(self.logger, "Transfer failed: {}", err);
                    counters::TX_FAILURE_COUNT.inc();
                    self.health_tracker.announce_failure();
                    match err {
                        TestClientError::ZeroBalance => {
                            counters::ZERO_BALANCE_COUNT.inc();
                        }
                        TestClientError::TxExpired => {
                            counters::TX_EXPIRED_COUNT.inc();
                        }
                        TestClientError::SubmittedTxTimeout => {
                            counters::CONFIRM_TX_TIMEOUT_COUNT.inc();
                        }
                        TestClientError::TxTimeout => {
                            counters::RECEIVE_TX_TIMEOUT_COUNT.inc();
                        }
                        TestClientError::BadBalance(_, _) => {
                            counters::BAD_BALANCE_COUNT.inc();
                        }
                        TestClientError::DoubleSpend => {
                            counters::TX_DOUBLE_SPEND_COUNT.inc();
                        }
                        TestClientError::UnexpectedMemo => {
                            counters::TX_UNEXPECTED_MEMO_COUNT.inc();
                        }
                        TestClientError::InvalidMemo => {
                            counters::TX_INVALID_MEMO_COUNT.inc();
                        }
                        TestClientError::CheckBalance(_) => {
                            counters::CHECK_BALANCE_ERROR_COUNT.inc();
                        }
                        TestClientError::BuildTx(_) => {
                            counters::BUILD_TX_ERROR_COUNT.inc();
                        }
                        TestClientError::SubmitTx(_) => {
                            counters::SUBMIT_TX_ERROR_COUNT.inc();
                        }
                        TestClientError::ConfirmTx(_) => {
                            counters::CONFIRM_TX_ERROR_COUNT.inc();
                        }
                    }
                }
            }

            ti += 1;
            self.health_tracker.set_counter(ti);
            std::thread::sleep(period);
        }
    }
}

/// Helper struct: A thread to check balance continuously on the target client
/// This allows us accurately measure both TX confirmation time and TX receipt
/// time, simultaneously
pub struct ReceiveTxWorker {
    /// Handle to worker thread which is blocking on target client getting the
    /// right balance, or an error
    join_handle: Option<JoinHandle<Result<(), TestClientError>>>,
    /// A flag to tell the worker thread to bail early because we failed
    bail: Arc<AtomicBool>,
    /// A "lazy option" with which we can tell the worker thread in what block
    /// the Tx landed, to help it detect if target client has failed.
    tx_appeared_relay: Arc<OnceCell<BlockIndex>>,
}

impl ReceiveTxWorker {
    /// Create and start a new Receive Tx worker thread
    ///
    /// Arguments:
    /// * client: The receiving client to check
    /// * current balance: The current balance of that client
    /// * expected balance: The expected balance after the Tx is received
    /// * policy: The test client policy object
    /// * expected_memo_contents: Optional short address hash matching the
    ///   sender's account
    /// * logger
    pub fn new(
        client: Arc<Mutex<Client>>,
        current_balance: u64,
        expected_balance: u64,
        policy: TestClientPolicy,
        expected_memo_contents: Option<ShortAddressHash>,
        tx_info: Arc<TxInfo>,
        health_tracker: Arc<HealthTracker>,
        logger: Logger,
        parent_context: Context,
    ) -> Self {
        let bail = Arc::new(AtomicBool::default());
        let tx_appeared_relay = Arc::new(OnceCell::<BlockIndex>::default());

        let thread_bail = bail.clone();
        let thread_relay = tx_appeared_relay.clone();

        let join_handle = Some(std::thread::spawn(
            move || -> Result<(), TestClientError> {
                let mut client = client.lock().expect("Could not lock client");
                let start = Instant::now();
                let mut deadline = Some(start + policy.tx_receive_deadline);

                let tracer = tracer!();
                let span = tracer
                    .span_builder("fog_view_received")
                    .with_kind(SpanKind::Server)
                    .with_parent_context(parent_context)
                    .start(&tracer);
                let _active = mark_span_as_active(span);

                loop {
                    if thread_bail.load(Ordering::SeqCst) {
                        return Ok(());
                    }

                    let (new_balance, new_block_count) = client
                        .check_balance()
                        .map_err(TestClientError::CheckBalance)?;

                    if new_balance == expected_balance {
                        counters::TX_RECEIVED_TIME.observe(start.elapsed().as_secs_f64());

                        if policy.test_rth_memos {
                            // Ensure target client got a sender memo, as expected for recoverable
                            // transcation history
                            match client.get_last_memo() {
                                Ok(Some(memo)) => match memo {
                                    MemoType::AuthenticatedSender(memo) => {
                                        if let Some(hash) = expected_memo_contents {
                                            if memo.sender_address_hash() != hash {
                                                log::error!(logger, "Target Client: Unexpected address hash: {:?} != {:?}. TxInfo: {}", memo.sender_address_hash(), hash, tx_info);
                                                return Err(TestClientError::UnexpectedMemo);
                                            }
                                        }
                                    }
                                    _ => {
                                        log::error!(
                                            logger,
                                            "Target Client: Unexpected memo type. TxInfo: {}",
                                            tx_info
                                        );
                                        return Err(TestClientError::UnexpectedMemo);
                                    }
                                },
                                Ok(None) => {
                                    log::error!(
                                        logger,
                                        "Target Client: Missing memo. TxInfo: {}",
                                        tx_info
                                    );
                                    return Err(TestClientError::UnexpectedMemo);
                                }
                                Err(err) => {
                                    log::error!(
                                        logger,
                                        "Target Client: Memo parse error: {}. TxInfo: {}",
                                        err,
                                        tx_info
                                    );
                                    return Err(TestClientError::InvalidMemo);
                                }
                            }
                        }
                        return Ok(());
                    } else if new_balance != current_balance {
                        return Err(TestClientError::BadBalance(expected_balance, new_balance));
                    }

                    if let Some(tx_appeared) = thread_relay.get() {
                        // If the other thread told us the Tx appeared in a certain block, and
                        // we are past that block and still don't have expected balance,
                        // then we have a bad balance and can bail out
                        if u64::from(new_block_count) > *tx_appeared {
                            return Err(TestClientError::BadBalance(expected_balance, new_balance));
                        }
                    }

                    deadline = if let Some(deadline) = deadline {
                        if Instant::now() > deadline {
                            counters::TX_RECEIVED_DEADLINE_EXCEEDED_COUNT.inc();
                            // Announce unhealthy status once the deadline is exceeded, even if we
                            // don't fail fast
                            health_tracker.announce_failure();
                            log::error!(
                                logger,
                                "TX receive deadline ({:?}) was exceeded: {}",
                                policy.tx_receive_deadline,
                                tx_info
                            );
                            if policy.fail_fast_on_deadline {
                                return Err(TestClientError::TxTimeout);
                            }
                            None
                        } else {
                            Some(deadline)
                        }
                    } else {
                        None
                    };

                    std::thread::sleep(policy.polling_wait);
                }
            },
        ));

        Self {
            bail,
            tx_appeared_relay,
            join_handle,
        }
    }

    /// Inform the worker thread in which block the transaction landed.
    /// This helps it to detect an error state in which that block already
    /// passed and we didn't find the money (perhaps fog is broken)
    ///
    /// Arguments:
    /// * index: The block index in which the Tx landed
    pub fn relay_tx_appeared(&mut self, index: BlockIndex) {
        self.tx_appeared_relay
            .set(index)
            .expect("value was already relayed");
    }

    /// Join the worker thread and return its error (or ok) status
    pub fn join(mut self) -> Result<(), TestClientError> {
        self.join_handle
            .take()
            .expect("Missing join handle")
            .join()
            .expect("Could not join worker thread")
    }
}

impl Drop for ReceiveTxWorker {
    fn drop(&mut self) {
        // This test is needed because the user may call join, which will then drop
        // self.
        if let Some(handle) = self.join_handle.take() {
            // We store bail as true in this case, because for instance, if submitting the
            // Tx failed, then the target client balance will never change.
            self.bail.store(true, Ordering::SeqCst);
            let _ = handle.join();
        }
    }
}

/// An object which tracks info about a Tx as it evolves, for logging context
/// in case of errors.
/// This is thread-safe so that we can share it with the receive worker
#[derive(Default, Debug)]
pub struct TxInfo {
    /// Lock on inner data
    inner: Mutex<TxInfoInner>,
}

#[derive(Default, Debug)]
struct TxInfoInner {
    /// The Tx which was submitted
    tx: Option<Tx>,
    /// The block cloud returned by propose_tx
    tx_propose_block_count: Option<u64>,
    /// The block in which the tx appeared
    tx_appeared: Option<BlockIndex>,
}

impl TxInfo {
    /// Clear the TxInfo
    pub fn clear(&self) {
        *self.inner.lock().unwrap() = Default::default();
    }

    /// Set the Tx that we are sending (immediately after it is built)
    pub fn set_tx(&self, tx: &Tx) {
        self.inner.lock().unwrap().tx = Some(tx.clone());
    }

    /// Set the block count returned by tx_propose (immediately after it is
    /// known)
    pub fn set_tx_propose_block_count(&self, count: u64) {
        self.inner.lock().unwrap().tx_propose_block_count = Some(count);
    }

    /// Set the index in which the tx appeared (immediately after it is known)
    pub fn set_tx_appeared_block_index(&self, index: BlockIndex) {
        self.inner.lock().unwrap().tx_appeared = Some(index);
    }
}

impl core::fmt::Display for TxInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let guard = self.inner.lock().unwrap();
        if let Some(proposed) = &guard.tx_propose_block_count {
            write!(
                f,
                "Proposed at block index ~{}, ",
                proposed.saturating_sub(1)
            )?;
        }
        if let Some(appeared) = &guard.tx_appeared {
            write!(f, "Appeared in block index {}, ", appeared)?;
        }
        if let Some(tx) = &guard.tx {
            write!(
                f,
                "TxOut public keys: [{}]",
                HexList(tx.prefix.outputs.iter().map(|x| x.public_key.as_bytes()))
            )?;
        }
        Ok(())
    }
}

/// Object which manages the LAST_POLLING_SUCCESSFUL gauge
///
/// * If a failure is observed, we are unhealthy (immediately)
/// * If no failure is observed for a long enough time, we are healthy again
///
/// The amount of time is the "healing_time" and it is expected to be set
/// to the number of clients, so that when we go around once in round-robin
/// fashion and all clients are successful, we are considered healed, and not
/// before that.
#[derive(Default)]
pub struct HealthTracker {
    // Set to i for the duration of the i'th transfer
    counter: AtomicUsize,
    // Whether we have observed any failure
    have_failure: AtomicBool,
    // The counter value during the most recent failure
    last_failure: AtomicUsize,
    // Healing time: How many successful transfers needed to forget a failure
    healing_time: usize,
}

impl HealthTracker {
    /// Make a new healthy tracker.
    /// Sets LAST_POLLING_SUCCESSFUL to true initially.
    ///
    /// Takes "healing time" which is the number of successful transfers before
    /// we consider ourselves healthy again
    pub fn new(healing_time: usize) -> Self {
        counters::LAST_POLLING_SUCCESSFUL.set(1);
        Self {
            healing_time,
            ..Default::default()
        }
    }

    /// Set the counter value, and maybe update healthy status
    pub fn set_counter(&self, counter: usize) {
        self.counter.store(counter, Ordering::SeqCst);
        // If:
        // * there is a failure
        // * the failure happened at least healing_time ago
        // then set ourselves healthy again
        if self.have_failure.load(Ordering::SeqCst)
            && self.last_failure.load(Ordering::SeqCst) + self.healing_time <= counter
        {
            counters::LAST_POLLING_SUCCESSFUL.set(1);
        }
    }

    /// Announce a failure, which will update the healthy status, and be tracked
    pub fn announce_failure(&self) {
        self.last_failure
            .store(self.counter.load(Ordering::SeqCst), Ordering::SeqCst);
        // Store have_failure only after writing to last_failure
        self.have_failure.store(true, Ordering::SeqCst);
        counters::LAST_POLLING_SUCCESSFUL.set(0);
    }
}