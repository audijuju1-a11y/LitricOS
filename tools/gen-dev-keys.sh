#!/usr/bin/env bash
# =============================================================================
# LitricOS — generate DEVELOPMENT signing keys
#
# Produces the Secure Boot and dm-verity key pairs that image/mkosi.conf refers
# to. Without them mkosi fails at the validation stage, which is why
# ci/lint-mkosi.py treats a missing key as an error rather than a warning.
#
#   keys/dev/secureboot.key / .crt   signs the Unified Kernel Image
#   keys/dev/verity.key     / .crt   signs the dm-verity root hash
#
# =============================================================================
# THESE KEYS ARE NOT SECRET AND MUST NEVER SIGN A RELEASE.
#
# They exist so that a developer can build and boot a verity-protected,
# Secure-Boot-signed image on their own machine without access to any release
# infrastructure. Anyone who clones this repository can regenerate an identical
# pair, so an image signed with them proves nothing about its origin.
#
# Production keys live in an HSM and are used only by the release pipeline. The
# split is deliberate and is enforced by keys/.gitignore: the dev directory is
# generated locally and never committed, so there is no path by which a
# development key reaches a user's machine as though it were a release key.
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
KEYS_DIR="${PROJECT_ROOT}/keys/dev"

# Reproducibility: certificates carry a notBefore date. Deriving it from
# SOURCE_DATE_EPOCH keeps two generations byte-identical, which matters because
# the certificate ends up inside the signed UKI and therefore inside the image
# hash we claim is reproducible.
: "${SOURCE_DATE_EPOCH:=1756166400}"
VALID_DAYS="${LITRIC_DEV_KEY_DAYS:-3650}"

log()  { printf '\033[1;32m[keys]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[keys]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[keys]\033[0m %s\n' "$*" >&2; exit 1; }

command -v openssl >/dev/null 2>&1 || die "openssl not found"

# Git Bash / MSYS2 rewrites any argument that looks like a POSIX path into a
# Windows path before the program sees it, which turns an X.509 subject of
# "/CN=..." into "C:/Program Files/Git/CN=...". The build host is Linux, where
# this does not happen, but developers work on Windows and a script that only
# works on one of them is a script that wastes someone an afternoon.
#
# The exclusion is scoped to "/CN=" and NOT set to "*". Excluding everything
# also stops the conversion of -keyout and -out, so the native Windows openssl
# then receives "/c/Users/..." and cannot open it. Both halves have to be right:
# the subject must be left alone, the file paths must still be converted.
case "${OSTYPE:-}" in
    msys*|cygwin*|win32) export MSYS2_ARG_CONV_EXCL='/CN=' ;;
esac

mkdir -p "$KEYS_DIR"
chmod 0700 "$KEYS_DIR" 2>/dev/null || true

# Refuse to clobber. Regenerating a Secure Boot key after enrolling it in
# firmware leaves a machine that will not boot, and silently doing that during
# an unrelated build would be a genuinely destructive surprise.
if [[ "${1:-}" != "--force" ]]; then
    complete=1
    for f in secureboot.key secureboot.crt verity.key verity.crt pcr.key pcr.crt; do
        [[ -s "${KEYS_DIR}/${f}" ]] || complete=0
    done
    if (( complete )); then
        log "keys already present and complete in ${KEYS_DIR} (pass --force to regenerate)"
        log "NOTE: regenerating invalidates any firmware enrolment of the old key."
        exit 0
    fi
    # A partial set means a previous run failed part-way. Clear it and start
    # over rather than leaving an unusable half-pair in place.
    if compgen -G "${KEYS_DIR}/*" > /dev/null 2>&1; then
        warn "incomplete key set found; regenerating from scratch"
        rm -f "${KEYS_DIR}"/secureboot.* "${KEYS_DIR}"/verity.* "${KEYS_DIR}"/pcr.*
    fi
fi

gen_pair() {
    local name="$1" cn="$2"
    log "generating ${name} (RSA 4096, ${VALID_DAYS} days)"

    # RSA 4096, not ECDSA: UEFI Secure Boot implementations in the field have
    # far better RSA coverage, and a key that some firmware cannot verify is
    # worse than a slightly larger one.
    openssl req -x509 -newkey rsa:4096 -sha256 \
        -keyout "${KEYS_DIR}/${name}.key" \
        -out    "${KEYS_DIR}/${name}.crt" \
        -days   "$VALID_DAYS" \
        -nodes \
        -subj   "/CN=${cn}/O=LitricOS Development/OU=NOT FOR RELEASE" \
        -addext "basicConstraints=critical,CA:FALSE" \
        -addext "keyUsage=critical,digitalSignature" \
        -addext "extendedKeyUsage=codeSigning" \
        || die "openssl failed generating ${name} (openssl's own error is above)"

    chmod 0600 "${KEYS_DIR}/${name}.key" 2>/dev/null || true
    chmod 0644 "${KEYS_DIR}/${name}.crt" 2>/dev/null || true
}

gen_pair secureboot "LitricOS Development Secure Boot Key"
gen_pair verity     "LitricOS Development Verity Key"
# The TPM2 expected-PCR signature is a THIRD, separate key. It signs the policy
# that says "these are the PCR values a healthy boot produces", which is what
# lets systemd-cryptenroll unseal the disk without a passphrase. It is not the
# Secure Boot key and must not be: the Secure Boot key authorises code, this one
# attests to a measurement, and conflating them would mean a leak of either
# compromises both.
gen_pair pcr        "LitricOS Development PCR Signing Key"

cat > "${KEYS_DIR}/README" <<'EOF'
DEVELOPMENT KEYS -- NOT FOR RELEASE

Regenerable by anyone with tools/gen-dev-keys.sh. An image signed with these
keys proves nothing about who built it. They exist only so a developer can boot
a Secure-Boot-signed, verity-protected image locally.

Never enrol these in the firmware of a machine you care about, and never use
them to sign anything a user will install.
EOF

# The whole point of the dev/production split is that these never leave the
# machine. Belt and braces: a .gitignore inside the directory itself, so the
# protection survives someone copying the directory elsewhere.
cat > "${PROJECT_ROOT}/keys/.gitignore" <<'EOF'
# Development keys are generated locally by tools/gen-dev-keys.sh and are never
# committed. A committed signing key -- even a development one -- is a key that
# will eventually be used by mistake.
dev/
*.key
*.pem
EOF

log "done:"
for f in secureboot verity pcr; do
    printf '  %-16s %s\n' "${f}.crt" "$(openssl x509 -in "${KEYS_DIR}/${f}.crt" -noout -subject 2>/dev/null | sed 's/^subject=//')"
done
warn "these keys are development keys and must never sign a release"
