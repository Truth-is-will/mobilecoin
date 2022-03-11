// Warning, this file is autogenerated by cbindgen. Don't modify this manually.

/**
 * This file is meant to be used as an aid when making changes to the manually written
 * `libmobilecoin.h` header by comparing it to the output of this cbindgen-autogenerated
 * file and checking for equivalency.
 */

#define LIB_MC_ERROR_CODE_UNKNOWN -1

#define LIB_MC_ERROR_CODE_PANIC -2

#define LIB_MC_ERROR_CODE_INVALID_INPUT 100

#define LIB_MC_ERROR_CODE_INVALID_OUTPUT 101

#define LIB_MC_ERROR_CODE_ATTESTATION_VERIFICATION_FAILED 200

#define LIB_MC_ERROR_CODE_AEAD 300

#define LIB_MC_ERROR_CODE_CIPHER 301

#define LIB_MC_ERROR_CODE_UNSUPPORTED_CRYPTO_BOX_VERSION 302

#define LIB_MC_ERROR_CODE_TRANSACTION_CRYPTO 400

#define LIB_MC_ERROR_CODE_FOG_PUBKEY 500

typedef struct AttestAke AttestAke;

typedef struct McFogResolver McFogResolver;

typedef struct McTransactionBuilderRing McTransactionBuilderRing;

typedef struct Option_TransactionBuilder_FogResolver Option_TransactionBuilder_FogResolver;

typedef struct Vec_u8 Vec_u8;

typedef struct Vec_u8 McData;

typedef struct McMutableBuffer {
  FfiMutPtr<uint8_t> buffer;
  size_t len;
} McMutableBuffer;

typedef struct McError {
  int error_code;
  FfiOwnedStr error_description;
} McError;

typedef MrEnclaveVerifier McMrEnclaveVerifier;

/**
 * This type is meant to be used as a parameter (or field of an another
 * parameter, etc) to C-FFI functions to code written in Rust. Objects of this
 * type are typically allocated on the foreign side of the FFI boundary and are
 * passed in to Rust via an `extern fn`-style function.
 *
 * The purpose of this type is to encapsulate unsafety within a type, such that
 * if this type were to be created solely in safe Rust, that it would contain
 * no unsafety. This is to say that, while this type performs unsafe operations
 * internally, in order for those unsafe operations to actually cause unsafety,
 * this type must have been created or otherwise manipulated from unsafe
 * code (typically either unsafe Rust or unsafe-by-definition foreign code).
 * Therefore, care must be taken when using this type from unsafe code (and
 * indeed it is intended to be used from unsafe code), but the same care does
 * not need to be taken in order to otherwise use it from safe code,
 * with the assumption that no preconditions were violated from unsafe code.
 */
typedef struct McBuffer {
  FfiRefPtr<uint8_t> buffer;
  size_t len;
} McBuffer;

typedef MrSignerVerifier McMrSignerVerifier;

typedef Verifier McVerifier;

typedef struct AttestAke McAttestAke;

/**
 * Transparent wrapper around a function pointer that accepts a context
 * argument and returns a `u64`, intended for use as a parameter to FFI
 * functions so that foreign code may provide a callback for generating random
 * numbers.
 *
 * This type has the exact memory layout as the C equivalent `uint64_t
 * (*)(void*)` function pointer.
 *
 * `null` is not considered a valid value.
 */
typedef uint64_t (*FfiCallbackRng)(void*);

typedef struct McRngCallback {
  FfiCallbackRng rng;
  FfiOptMutPtr<void> context;
} McRngCallback;

typedef FullyValidatedFogPubkey McFullyValidatedFogPubkey;

typedef struct McPublicAddressFogInfo {
  FfiStr report_url;
  FfiStr report_id;
  FfiRefPtr<McBuffer> authority_sig;
} McPublicAddressFogInfo;

typedef struct McPublicAddress {
  /**
   * 32-byte `CompressedRistrettoPublic`
   */
  FfiRefPtr<McBuffer> view_public_key;
  /**
   * 32-byte `CompressedRistrettoPublic`
   */
  FfiRefPtr<McBuffer> spend_public_key;
  FfiOptRefPtr<McPublicAddressFogInfo> fog_info;
} McPublicAddress;

typedef VersionedKexRng McFogRng;

typedef struct McAccountKeyFogInfo {
  FfiStr report_url;
  FfiStr report_id;
  FfiRefPtr<McBuffer> authority_spki;
} McAccountKeyFogInfo;

typedef struct McAccountKey {
  /**
   * 32-byte `RistrettoPrivate`
   */
  FfiRefPtr<McBuffer> view_private_key;
  /**
   * 32-byte `RistrettoPrivate`
   */
  FfiRefPtr<McBuffer> spend_private_key;
  FfiOptRefPtr<McAccountKeyFogInfo> fog_info;
} McAccountKey;

typedef struct McTxOutAmount {
  /**
   * 32-byte `CompressedCommitment`
   */
  uint64_t masked_value;
} McTxOutAmount;

typedef struct Option_TransactionBuilder_FogResolver McTransactionBuilder;

void mc_data_free(FfiOptOwnedPtr<McData> data);

/**
 * # Preconditions
 *
 * * `out_bytes` - must be null or else length must be >= `data.len`.
 */
ssize_t mc_data_get_bytes(FfiRefPtr<McData> data, FfiOptMutPtr<McMutableBuffer> out_bytes);

/**
 * All non-null owned pointers of type `McError *` that are returned from a
 * Rust FFI function to a foreign caller must call this function in order to
 * free the underlying memory pointed to by the pointer.
 *
 * It is undefined behavior for foreign code to dereference the pointer after
 * it has called this method.
 */
void mc_error_free(FfiOptOwnedPtr<McError> error);

/**
 * All non-null values with a `char *` return (or out parameter) type that are
 * returned to foreign code must call this function in order to free the
 * underlying memory pointed to by the pointer.
 *
 * It is undefined behavior for foreign code to dereference the pointer after
 * it has called this method.
 */
void mc_string_free(FfiOptOwnedStr string);

void mc_mr_enclave_verifier_free(FfiOptOwnedPtr<McMrEnclaveVerifier> mr_enclave_verifier);

/**
 * Create a new status verifier that will check for the existence of the
 * given MrEnclave.
 *
 * # Preconditions
 *
 * * `mr_enclave` - must be 32 bytes in length.
 */
FfiOptOwnedPtr<McMrEnclaveVerifier> mc_mr_enclave_verifier_create(FfiRefPtr<McBuffer> mr_enclave);

/**
 * Assume an enclave with the specified measurement does not need
 * BIOS configuration changes to address the provided advisory ID.
 *
 * This method should only be used when advised by an enclave author.
 *
 * # Preconditions
 *
 * * `advisory_id` - must be a nul-terminated C string containing valid UTF-8.
 */
bool mc_mr_enclave_verifier_allow_config_advisory(FfiMutPtr<McMrEnclaveVerifier> mr_enclave_verifier,
                                                  FfiStr advisory_id);

/**
 * Assume the given MrEnclave value has the appropriate software/build-time
 * hardening for the given advisory ID.
 *
 * This method should only be used when advised by an enclave author.
 *
 * # Preconditions
 *
 * * `advisory_id` - must be a nul-terminated C string containing valid UTF-8.
 */
bool mc_mr_enclave_verifier_allow_hardening_advisory(FfiMutPtr<McMrEnclaveVerifier> mr_enclave_verifier,
                                                     FfiStr advisory_id);

void mc_mr_signer_verifier_free(FfiOptOwnedPtr<McMrSignerVerifier> mr_signer_verifier);

/**
 * Create a new status verifier that will check for the existence of the
 * given MrSigner.
 *
 * # Preconditions
 *
 * * `mr_signer` - must be 32 bytes in length.
 */
FfiOptOwnedPtr<McMrSignerVerifier> mc_mr_signer_verifier_create(FfiRefPtr<McBuffer> mr_signer,
                                                                uint16_t expected_product_id,
                                                                uint16_t minimum_security_version);

/**
 * Assume an enclave with the specified measurement does not need
 * BIOS configuration changes to address the provided advisory ID.
 *
 * This method should only be used when advised by an enclave author.
 *
 * # Preconditions
 *
 * * `advisory_id` - must be a nul-terminated C string containing valid UTF-8.
 */
bool mc_mr_signer_verifier_allow_config_advisory(FfiMutPtr<MrSignerVerifier> mr_signer_verifier,
                                                 FfiStr advisory_id);

/**
 * Assume an enclave with the specified measurement has the appropriate
 * software/build-time hardening for the given advisory ID.
 *
 * This method should only be used when advised by an enclave author.
 *
 * # Preconditions
 *
 * * `advisory_id` - must be a nul-terminated C string containing valid UTF-8.
 */
bool mc_mr_signer_verifier_allow_hardening_advisory(FfiMutPtr<MrSignerVerifier> mr_signer_verifier,
                                                    FfiStr advisory_id);

/**
 * Construct a new builder using the baked-in IAS root certificates and debug
 * settings.
 */
FfiOptOwnedPtr<McVerifier> mc_verifier_create(void);

void mc_verifier_free(FfiOptOwnedPtr<McVerifier> verifier);

/**
 * Verify the given MrEnclave-based status verifier succeeds
 */
bool mc_verifier_add_mr_enclave(FfiMutPtr<McVerifier> verifier,
                                FfiRefPtr<McMrEnclaveVerifier> mr_enclave_verifier);

/**
 * Verify the given MrSigner-based status verifier succeeds
 */
bool mc_verifier_add_mr_signer(FfiMutPtr<McVerifier> verifier,
                               FfiRefPtr<McMrSignerVerifier> mr_signer_verifier);

FfiOptOwnedPtr<McAttestAke> mc_attest_ake_create(void);

void mc_attest_ake_free(FfiOptOwnedPtr<McAttestAke> attest_ake);

bool mc_attest_ake_is_attested(FfiRefPtr<McAttestAke> attest_ake, FfiMutPtr<bool> out_attested);

/**
 * # Preconditions
 *
 * * `attest_ake` - must be in the attested state.
 * * `out_binding` - must be null or else length must be >= `binding.len`.
 */
ssize_t mc_attest_ake_get_binding(FfiRefPtr<McAttestAke> attest_ake,
                                  FfiOptMutPtr<McMutableBuffer> out_binding);

/**
 * # Preconditions
 *
 * * `responder_id` - must be a nul-terminated C string containing a valid
 *   responder ID.
 * * `out_auth_request` - must be null or else length must be >=
 *   auth_request_output.len.
 */
ssize_t mc_attest_ake_get_auth_request(FfiMutPtr<McAttestAke> attest_ake,
                                       FfiStr responder_id,
                                       FfiOptMutPtr<McRngCallback> rng_callback,
                                       FfiOptMutPtr<McMutableBuffer> out_auth_request);

/**
 * # Preconditions
 *
 * * `attest_ake` - must be in the auth pending state.
 *
 * # Errors
 *
 * * `LibMcError::AttestationVerificationFailed`
 * * `LibMcError::InvalidInput`
 */
bool mc_attest_ake_process_auth_response(FfiMutPtr<McAttestAke> attest_ake,
                                         FfiRefPtr<McBuffer> auth_response_data,
                                         FfiRefPtr<McVerifier> verifier,
                                         FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `attest_ake` - must be in the attested state.
 * * `out_ciphertext` - must be null or else length must be >=
 *   `ciphertext.len`.
 *
 * # Errors
 *
 * * `LibMcError::Aead`
 * * `LibMcError::Cipher`
 */
ssize_t mc_attest_ake_encrypt(FfiMutPtr<McAttestAke> attest_ake,
                              FfiRefPtr<McBuffer> aad,
                              FfiRefPtr<McBuffer> plaintext,
                              FfiOptMutPtr<McMutableBuffer> out_ciphertext,
                              FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `attest_ake` - must be in the attested state.
 * * `out_plaintext` - length must be >= `ciphertext.len`.
 *
 * # Errors
 *
 * * `LibMcError::Aead`
 * * `LibMcError::Cipher`
 */
ssize_t mc_attest_ake_decrypt(FfiMutPtr<McAttestAke> attest_ake,
                              FfiRefPtr<McBuffer> aad,
                              FfiRefPtr<McBuffer> ciphertext,
                              FfiMutPtr<McMutableBuffer> out_plaintext,
                              FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `entropy` - length must be a multiple of 4 and between 16 and 32,
 *   inclusive, in bytes.
 */
FfiOptOwnedStr mc_bip39_mnemonic_from_entropy(FfiRefPtr<McBuffer> entropy);

/**
 * # Preconditions
 *
 * * `mnemonic` - must be a nul-terminated C string containing valid UTF-8.
 * * `out_entropy` - must be null or else length must be >= `entropy.len`.
 *
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 */
ssize_t mc_bip39_entropy_from_mnemonic(FfiStr mnemonic,
                                       FfiOptMutPtr<McMutableBuffer> out_entropy,
                                       FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `prefix` - must be a nul-terminated C string containing valid UTF-8.
 */
FfiOptOwnedStr mc_bip39_words_by_prefix(FfiStr prefix);

bool mc_ristretto_private_validate(FfiRefPtr<McBuffer> ristretto_private,
                                   FfiMutPtr<bool> out_valid);

/**
 * # Preconditions
 *
 * * `ristretto_private` - must be a valid 32-byte Ristretto-format scalar.
 * * `out_ristretto_public` - length must be >= 32.
 */
bool mc_ristretto_public_from_ristretto_private(FfiRefPtr<McBuffer> ristretto_private,
                                                FfiMutPtr<McMutableBuffer> out_ristretto_public);

bool mc_ristretto_public_validate(FfiRefPtr<McBuffer> ristretto_public, FfiMutPtr<bool> out_valid);

/**
 * # Preconditions
 *
 * * `public_key` - must be a valid 32-byte compressed Ristretto point.
 * * `out_ciphertext` - must be null or else length must be >=
 *   `ciphertext.len`.
 *
 * # Errors
 *
 * * `LibMcError::Aead`
 */
ssize_t mc_versioned_crypto_box_encrypt(FfiRefPtr<McBuffer> public_key,
                                        FfiRefPtr<McBuffer> plaintext,
                                        FfiOptMutPtr<McRngCallback> rng_callback,
                                        FfiOptMutPtr<McMutableBuffer> out_ciphertext,
                                        FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `private_key` - must be a valid 32-byte Ristretto-format scalar.
 * * `out_plaintext` - length must be >= `ciphertext.len`.
 *
 * # Errors
 *
 * * `LibMcError::Aead`
 * * `LibMcError::InvalidInput`
 * * `LibMcError::UnsupportedCryptoBoxVersion`
 */
ssize_t mc_versioned_crypto_box_decrypt(FfiRefPtr<McBuffer> private_key,
                                        FfiRefPtr<McBuffer> ciphertext,
                                        FfiMutPtr<McMutableBuffer> out_plaintext,
                                        FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `printable_wrapper_proto_bytes` - must be a valid binary-serialized
 *   `printable.PrintableWrapper` Protobuf.
 */
FfiOptOwnedStr mc_printable_wrapper_b58_encode(FfiRefPtr<McBuffer> printable_wrapper_proto_bytes);

/**
 * # Preconditions
 *
 * * `b58_encoded_string` - must be a nul-terminated C string containing valid
 *   UTF-8.
 * * `out_printable_wrapper_proto_bytes` - must be null or else length must be
 *   >= `wrapper_bytes.len`.
 *
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 */
ssize_t mc_printable_wrapper_b58_decode(FfiStr b58_encoded_string,
                                        FfiOptMutPtr<McMutableBuffer> out_printable_wrapper_proto_bytes,
                                        FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

FfiOptOwnedPtr<McFogResolver> mc_fog_resolver_create(FfiRefPtr<McVerifier> fog_report_verifier);

void mc_fog_resolver_free(FfiOptOwnedPtr<McFogResolver> fog_resolver);

FfiOptOwnedPtr<McFullyValidatedFogPubkey> mc_fog_resolver_get_fog_pubkey(FfiRefPtr<McFogResolver> fog_resolver,
                                                                         FfiRefPtr<McPublicAddress> recipient,
                                                                         FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

FfiOptOwnedPtr<McFullyValidatedFogPubkey> mc_fog_resolver_get_fog_pubkey_from_protobuf_public_address(FfiRefPtr<McFogResolver> fog_resolver,
                                                                                                      FfiRefPtr<McBuffer> recipient_protobuf,
                                                                                                      FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `report_url` - must be a nul-terminated C string containing a valid Fog
 *   report uri.
 *
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 */
bool mc_fog_resolver_add_report_response(FfiMutPtr<McFogResolver> fog_resolver,
                                         FfiStr report_url,
                                         FfiRefPtr<McBuffer> report_response,
                                         FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

void mc_fully_validated_fog_pubkey_free(FfiOptOwnedPtr<McFullyValidatedFogPubkey> fully_validated_fog_pubkey);

void mc_fully_validated_fog_pubkey_get_pubkey(FfiRefPtr<McFullyValidatedFogPubkey> fully_validated_fog_pubkey,
                                              FfiMutPtr<McMutableBuffer> out_pubkey);

uint64_t mc_fully_validated_fog_pubkey_get_pubkey_expiry(FfiRefPtr<McFullyValidatedFogPubkey> fully_validated_fog_pubkey);

/**
 * # Preconditions
 *
 * * `subaddress_view_private_key` - must be a valid 32-byte Ristretto-format
 *   scalar.
 *
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 * * `LibMcError::UnsupportedCryptoBoxVersion`
 */
FfiOptOwnedPtr<McFogRng> mc_fog_rng_create(FfiRefPtr<McBuffer> subaddress_view_private_key,
                                           FfiRefPtr<McBuffer> rng_public_key,
                                           uint32_t rng_version,
                                           FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

void mc_fog_rng_free(FfiOptOwnedPtr<McFogRng> fog_rng);

FfiOptOwnedPtr<McFogRng> mc_fog_rng_clone(FfiRefPtr<McFogRng> fog_rng);

/**
 * # Preconditions
 *
 * * `out_fog_rng_proto_bytes` - must be null or else length must be >=
 *   `encoded.len`.
 */
ssize_t mc_fog_rng_serialize_proto(FfiRefPtr<McFogRng> fog_rng,
                                   FfiOptMutPtr<McMutableBuffer> out_fog_rng_proto_bytes);

/**
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 * * `LibMcError::UnsupportedCryptoBoxVersion`
 */
FfiOptOwnedPtr<McFogRng> mc_fog_rng_deserialize_proto(FfiRefPtr<McBuffer> fog_rng_proto_bytes,
                                                      FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

int64_t mc_fog_rng_index(FfiRefPtr<McFogRng> fog_rng);

ssize_t mc_fog_rng_get_output_len(FfiRefPtr<McFogRng> fog_rng);

/**
 * # Preconditions
 *
 * * `out_output` - length must be >= `output.len`.
 */
bool mc_fog_rng_peek(FfiRefPtr<McFogRng> fog_rng, FfiMutPtr<McMutableBuffer> out_output);

/**
 * # Preconditions
 *
 * * `out_output` - must be null or else length must be >= `output.len`.
 */
bool mc_fog_rng_advance(FfiMutPtr<McFogRng> fog_rng, FfiOptMutPtr<McMutableBuffer> out_output);

/**
 * # Preconditions
 *
 * * `view_private_key` - must be a valid 32-byte Ristretto-format scalar.
 * * `spend_private_key` - must be a valid 32-byte Ristretto-format scalar.
 * * `out_subaddress_view_private_key` - length must be >= 32.
 * * `out_subaddress_spend_private_key` - length must be >= 32.
 */
bool mc_account_key_get_subaddress_private_keys(FfiRefPtr<McBuffer> view_private_key,
                                                FfiRefPtr<McBuffer> spend_private_key,
                                                uint64_t subaddress_index,
                                                FfiMutPtr<McMutableBuffer> out_subaddress_view_private_key,
                                                FfiMutPtr<McMutableBuffer> out_subaddress_spend_private_key);

/**
 * # Preconditions
 *
 * * `view_private_key` - must be a valid 32-byte Ristretto-format scalar.
 * * `spend_private_key` - must be a valid 32-byte Ristretto-format scalar.
 * * `out_subaddress_view_public_key` - length must be >= 32.
 * * `out_subaddress_spend_public_key` - length must be >= 32.
 */
bool mc_account_key_get_public_address_public_keys(FfiRefPtr<McBuffer> view_private_key,
                                                   FfiRefPtr<McBuffer> spend_private_key,
                                                   uint64_t subaddress_index,
                                                   FfiMutPtr<McMutableBuffer> out_subaddress_view_public_key,
                                                   FfiMutPtr<McMutableBuffer> out_subaddress_spend_public_key);

/**
 * # Preconditions
 *
 * * `account_key` - must be a valid `AccountKey` with `fog_info`.
 * * `out_fog_authority_sig` - length must be >= 64.
 */
bool mc_account_key_get_public_address_fog_authority_sig(FfiRefPtr<McAccountKey> account_key,
                                                         uint64_t subaddress_index,
                                                         FfiMutPtr<McMutableBuffer> out_fog_authority_sig);

/**
 * # Preconditions
 *
 * * `mnemonic` - must be a nul-terminated C string containing valid UTF-8.
 * * `out_view_private_key` - length must be >= 32.
 * * `out_spend_private_key` - length must be >= 32.
 *
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 */
bool mc_slip10_account_private_keys_from_mnemonic(FfiStr mnemonic,
                                                  uint32_t account_index,
                                                  FfiMutPtr<McMutableBuffer> out_view_private_key,
                                                  FfiMutPtr<McMutableBuffer> out_spend_private_key,
                                                  FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `view_private_key` - must be a valid 32-byte Ristretto-format scalar.
 */
bool mc_tx_out_reconstruct_commitment(FfiRefPtr<McTxOutAmount> tx_out_amount,
                                      FfiRefPtr<McBuffer> tx_out_public_key,
                                      FfiRefPtr<McBuffer> view_private_key,
                                      FfiMutPtr<McMutableBuffer> out_tx_out_commitment,
                                      FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `tx_out_commitment` - must be a valid CompressedCommitment
 *
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 */
bool mc_tx_out_commitment_crc32(FfiRefPtr<McBuffer> tx_out_commitment,
                                FfiMutPtr<uint32_t> out_crc32,
                                FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `view_private_key` - must be a valid 32-byte Ristretto-format scalar.
 */
bool mc_tx_out_matches_any_subaddress(FfiRefPtr<McTxOutAmount> _tx_out_amount,
                                      FfiRefPtr<McBuffer> tx_out_public_key,
                                      FfiRefPtr<McBuffer> view_private_key,
                                      FfiMutPtr<bool> out_matches);

/**
 * # Preconditions
 *
 * * `view_private_key` - must be a valid 32-byte Ristretto-format scalar.
 * * `subaddress_spend_private_key` - must be a valid 32-byte Ristretto-format
 *   scalar.
 */
bool mc_tx_out_matches_subaddress(FfiRefPtr<McBuffer> tx_out_target_key,
                                  FfiRefPtr<McBuffer> tx_out_public_key,
                                  FfiRefPtr<McBuffer> view_private_key,
                                  FfiRefPtr<McBuffer> subaddress_spend_private_key,
                                  FfiMutPtr<bool> out_matches);

/**
 * # Preconditions
 *
 * * `view_private_key` - must be a valid 32-byte Ristretto-format scalar.
 * * `out_subaddress_spend_public_key` - length must be >= 32.
 *
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 */
bool mc_tx_out_get_subaddress_spend_public_key(FfiRefPtr<McBuffer> tx_out_target_key,
                                               FfiRefPtr<McBuffer> tx_out_public_key,
                                               FfiRefPtr<McBuffer> view_private_key,
                                               FfiMutPtr<McMutableBuffer> out_subaddress_spend_public_key,
                                               FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `view_private_key` - must be a valid 32-byte Ristretto-format scalar.
 *
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 * * `LibMcError::TransactionCrypto`
 */
bool mc_tx_out_get_value(FfiRefPtr<McTxOutAmount> tx_out_amount,
                         FfiRefPtr<McBuffer> tx_out_public_key,
                         FfiRefPtr<McBuffer> view_private_key,
                         FfiMutPtr<uint64_t> out_value,
                         FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `view_private_key` - must be a valid 32-byte Ristretto-format scalar.
 * * `subaddress_spend_private_key` - must be a valid 32-byte Ristretto-format
 *   scalar.
 * * `out_key_image` - length must be >= 32.
 *
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 * * `LibMcError::TransactionCrypto`
 */
bool mc_tx_out_get_key_image(FfiRefPtr<McBuffer> tx_out_target_key,
                             FfiRefPtr<McBuffer> tx_out_public_key,
                             FfiRefPtr<McBuffer> view_private_key,
                             FfiRefPtr<McBuffer> subaddress_spend_private_key,
                             FfiMutPtr<McMutableBuffer> out_key_image,
                             FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `view_private_key` - must be a valid 32-byte Ristretto-format scalar.
 */
bool mc_tx_out_validate_confirmation_number(FfiRefPtr<McBuffer> tx_out_public_key,
                                            FfiRefPtr<McBuffer> tx_out_confirmation_number,
                                            FfiRefPtr<McBuffer> view_private_key,
                                            FfiMutPtr<bool> out_valid);

FfiOptOwnedPtr<McTransactionBuilderRing> mc_transaction_builder_ring_create(void);

void mc_transaction_builder_ring_free(FfiOptOwnedPtr<McTransactionBuilderRing> transaction_builder_ring);

/**
 * # Preconditions
 *
 * * `tx_out_proto_bytes` - must be a valid binary-serialized `external.TxOut`
 *   Protobuf.
 * * `membership_proof_proto_bytes` - must be a valid binary-serialized
 *   `external.TxOutMembershipProof` Protobuf.
 */
bool mc_transaction_builder_ring_add_element(FfiMutPtr<McTransactionBuilderRing> ring,
                                             FfiRefPtr<McBuffer> tx_out_proto_bytes,
                                             FfiRefPtr<McBuffer> membership_proof_proto_bytes);

FfiOptOwnedPtr<McTransactionBuilder> mc_transaction_builder_create(uint64_t fee,
                                                                   uint64_t tombstone_block,
                                                                   FfiOptRefPtr<McFogResolver> fog_resolver);

void mc_transaction_builder_free(FfiOptOwnedPtr<McTransactionBuilder> transaction_builder);

/**
 * # Preconditions
 *
 * * `transaction_builder` - must not have been previously consumed by a call
 *   to `build`.
 * * `view_private_key` - must be a valid 32-byte Ristretto-format scalar.
 * * `subaddress_spend_private_key` - must be a valid 32-byte Ristretto-format
 *   scalar.
 * * `real_index` - must be within bounds of `ring`.
 * * `ring` - `TxOut` at `real_index` must be owned by account keys.
 *
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 */
bool mc_transaction_builder_add_input(FfiMutPtr<McTransactionBuilder> transaction_builder,
                                      FfiRefPtr<McBuffer> view_private_key,
                                      FfiRefPtr<McBuffer> subaddress_spend_private_key,
                                      uintptr_t real_index,
                                      FfiRefPtr<McTransactionBuilderRing> ring,
                                      FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `transaction_builder` - must not have been previously consumed by a call
 *   to `build`.
 * * `recipient_address` - must be a valid `PublicAddress`.
 * * `out_subaddress_spend_public_key` - length must be >= 32.
 *
 * # Errors
 *
 * * `LibMcError::AttestationVerification`
 * * `LibMcError::InvalidInput`
 */
FfiOptOwnedPtr<McData> mc_transaction_builder_add_output(FfiMutPtr<McTransactionBuilder> transaction_builder,
                                                         uint64_t amount,
                                                         FfiRefPtr<McPublicAddress> recipient_address,
                                                         FfiOptMutPtr<McRngCallback> rng_callback,
                                                         FfiMutPtr<McMutableBuffer> out_tx_out_confirmation_number,
                                                         FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);

/**
 * # Preconditions
 *
 * * `transaction_builder` - must not have been previously consumed by a call
 *   to `build`.
 * * `recipient_address` - must be a valid `PublicAddress`.
 * * `fog_hint_address` - must be a valid `PublicAddress` with `fog_info`.
 * * `out_tx_out_confirmation_number` - length must be >= 32.
 *
 * # Errors
 *
 * * `LibMcError::AttestationVerification`
 * * `LibMcError::InvalidInput`
 */
FfiOptOwnedPtr<McData> mc_transaction_builder_add_output_with_fog_hint_address(FfiMutPtr<McTransactionBuilder> _transaction_builder,
                                                                               uint64_t _amount,
                                                                               FfiRefPtr<McPublicAddress> _recipient_address,
                                                                               FfiRefPtr<McPublicAddress> _fog_hint_address,
                                                                               FfiOptMutPtr<McRngCallback> _rng_callback,
                                                                               FfiMutPtr<McMutableBuffer> _out_tx_out_confirmation_number,
                                                                               FfiOptMutPtr<FfiOptOwnedPtr<McError>> _out_error);

/**
 * # Preconditions
 *
 * * `transaction_builder` - must not have been previously consumed by a call
 *   to `build`.
 *
 * # Errors
 *
 * * `LibMcError::InvalidInput`
 */
FfiOptOwnedPtr<McData> mc_transaction_builder_build(FfiMutPtr<McTransactionBuilder> transaction_builder,
                                                    FfiOptMutPtr<McRngCallback> rng_callback,
                                                    FfiOptMutPtr<FfiOptOwnedPtr<McError>> out_error);
