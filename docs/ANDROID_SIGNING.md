# Android APK Signing Configuration

## Problem

Debug APKs are signed with auto-generated debug keys that differ between machines. This causes "app not installed" errors when trying to update an installed app with an APK signed by a different debug key.

## Solution

Use a single release keystore for all production APKs distributed via GitHub releases.

---

## Setup Instructions

### 1. Generate Release Keystore (One-time setup)

Run this command to create your signing key:

```bash
keytool -genkey -v \
  -keystore potasko-release.jks \
  -alias potasko \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000 \
  -storepass <STORE_PASSWORD> \
  -keypass <KEY_PASSWORD>
```

**IMPORTANT:**
- Save the passwords securely (you'll need them for GitHub secrets)
- Keep the `.jks` file safe - losing it means you can't update your app
- **Never commit the `.jks` file to git** (already in `.gitignore`)

**Fill in these details when prompted:**
- First and last name: Your name or organization
- Organizational unit: Your team/company
- Organization: Your company name
- City/Locality: Your city
- State/Province: Your state
- Country code: Two-letter country code (e.g., US, UK, PL)

### 2. Add Keystore to GitHub Secrets

Base64 encode the keystore:
```bash
base64 -w 0 potasko-release.jks > potasko-release.jks.b64
```

Add these secrets to your GitHub repository (Settings → Secrets and variables → Actions):

| Secret Name | Value |
|-------------|-------|
| `ANDROID_KEYSTORE_BASE64` | Contents of `potasko-release.jks.b64` |
| `ANDROID_KEYSTORE_PASSWORD` | Your store password |
| `ANDROID_KEY_ALIAS` | `potasko` |
| `ANDROID_KEY_PASSWORD` | Your key password |

### 3. Build Configuration

The following files have been configured to support release signing:

- **`.github/workflows/android.yml`** - Decodes keystore and builds signed release APKs
- **`src-tauri/gen/android/app/build.gradle.kts`** - Signing configuration for Gradle

---

## Local Development

For local development, you can continue using debug builds:
```bash
cargo tauri android build --debug
```

To test release builds locally:
```bash
# Create local keystore.properties file (DO NOT COMMIT)
cat > src-tauri/gen/android/keystore.properties << EOF
storeFile=../../potasko-release.jks
storePassword=<YOUR_STORE_PASSWORD>
keyAlias=potasko
keyPassword=<YOUR_KEY_PASSWORD>
EOF

# Build release APK
cargo tauri android build --release --target aarch64
```

---

## Verifying Signing

Check which key signed an APK:
```bash
apksigner verify --print-certs app-release.apk
```

Both the old and new APK should show the same certificate fingerprint (SHA-256) if signed with the same key.

---

## Key Rotation (Advanced)

If you need to rotate your signing key:
1. Generate a new keystore (as above)
2. Update GitHub secrets with new values
3. **Important:** Users will need to uninstall the old app before installing the new one
4. Consider migrating to Google Play App Signing to avoid this in the future

---

## Security Notes

- **Never commit** `keystore.properties` or `.jks` files to git
- Treat the keystore like a password - losing it means you can't update your app
- Consider storing a backup of the keystore in a secure location (password manager, encrypted backup)
- For Play Store distribution, consider using Google Play App Signing for additional security
