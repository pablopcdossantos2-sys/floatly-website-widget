$ErrorActionPreference = "Stop"

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][scriptblock]$Command
    )
    Write-Host ""
    Write-Host "== $Label ==" -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE."
    }
}

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot

$pfxPath = $null
$importedThumbprints = @()
$signtool = $null
$runtimeConfigPath = $null
$storeExtractDir = $null

try {
    if ([string]::IsNullOrWhiteSpace($env:WINDOWS_PFX_BASE64) -or
        [string]::IsNullOrWhiteSpace($env:WINDOWS_PFX_PASSWORD)) {
        throw "Set WINDOWS_PFX_BASE64 and WINDOWS_PFX_PASSWORD to a trusted code-signing PFX before building the Microsoft Store installer."
    }

    if (!(Test-Path "node_modules/@tauri-apps/cli")) {
        throw "Dependencies are missing. Run npm ci from the repository root first."
    }

    $tauriConfig = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
    if ([string]::IsNullOrWhiteSpace($tauriConfig.bundle.publisher)) {
        throw "Set bundle.publisher in src-tauri/tauri.conf.json to the real publisher identity before creating the Store installer."
    }

    Invoke-Checked "Validate Store-specific project configuration" { npm run check:store-config }
    Invoke-Checked "Check Tauri command permissions" { npm run check:acl }
    Invoke-Checked "Check drag overlay behavior" { npm run check:drag-overlay }
    Invoke-Checked "Check dependency versions" { npm run check:tauri-versions }
    Invoke-Checked "Type-check the frontend" { npm run typecheck }
    Invoke-Checked "Audit npm dependencies" { npm audit --audit-level=high }
    Invoke-Checked "Check Rust formatting" { cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check }
    Invoke-Checked "Run Rust tests" { cargo test --manifest-path src-tauri/Cargo.toml --locked }
    Invoke-Checked "Run Clippy" { cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings }
    Invoke-Checked "Generate application icons" { npm run icons }

    Remove-Item -Recurse -Force "src-tauri/target/release/bundle" -ErrorAction SilentlyContinue
    Remove-Item -Force "src-tauri/target/release/widget-website.exe" -ErrorAction SilentlyContinue

    # Compile the app without bundling. It must be signed before NSIS embeds it.
    Invoke-Checked "Compile the Windows application without bundling" { npm run tauri build -- --no-bundle }

    $appBinary = Join-Path $repoRoot "src-tauri/target/release/widget-website.exe"
    if (!(Test-Path $appBinary)) {
        throw "The application executable was not generated: $appBinary"
    }

    $pfxPath = Join-Path ([System.IO.Path]::GetTempPath()) ("widget-website-store-" + [guid]::NewGuid().ToString("N") + ".pfx")
    $pfxBytes = [Convert]::FromBase64String(($env:WINDOWS_PFX_BASE64 -replace '\s', ''))
    [System.IO.File]::WriteAllBytes($pfxPath, $pfxBytes)
    $securePassword = ConvertTo-SecureString -String $env:WINDOWS_PFX_PASSWORD -AsPlainText -Force

    $existingThumbprints = @(Get-ChildItem Cert:\CurrentUser\My | ForEach-Object { $_.Thumbprint })
    $certificates = @(Import-PfxCertificate -FilePath $pfxPath -CertStoreLocation Cert:\CurrentUser\My -Password $securePassword)
    $certificate = $certificates | Where-Object { $_.HasPrivateKey } | Select-Object -First 1

    if ($null -eq $certificate) {
        throw "The PFX did not contain a certificate with a private key."
    }
    if ($certificate.NotBefore -gt (Get-Date) -or $certificate.NotAfter -le (Get-Date)) {
        throw "The code-signing certificate is not currently valid."
    }

    $ekuExtension = $certificate.Extensions |
        Where-Object { $_.Oid.Value -eq "2.5.29.37" } |
        Select-Object -First 1
    $ekuOids = @()
    if ($null -ne $ekuExtension) {
        $ekuOids = @($ekuExtension.EnhancedKeyUsages | ForEach-Object { $_.Value })
    }
    if ($ekuOids -notcontains "1.3.6.1.5.5.7.3.3") {
        throw "The selected certificate does not explicitly include the Code Signing EKU."
    }

    $importedThumbprints = @(
        $certificates |
        Where-Object { $existingThumbprints -notcontains $_.Thumbprint } |
        ForEach-Object { $_.Thumbprint }
    )

    $sdkBin = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"
    $signtool = Get-ChildItem $sdkBin -Recurse -Filter "signtool.exe" -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Directory.Name -eq "x64" } |
        Sort-Object {
            try { [version]$_.Directory.Parent.Name }
            catch { [version]"0.0" }
        } -Descending |
        Select-Object -First 1

    if ($null -eq $signtool) {
        throw "Windows SDK signtool.exe was not found. Install the Windows 10/11 SDK and retry."
    }

    function Sign-And-Verify {
        param([Parameter(Mandatory = $true)][string]$Path)

        Write-Host "Signing: $Path"
        & $signtool.FullName sign /s My /sha1 $certificate.Thumbprint /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $Path
        if ($LASTEXITCODE -ne 0) {
            throw "signtool failed to sign $Path."
        }

        & $signtool.FullName verify /pa /v $Path
        if ($LASTEXITCODE -ne 0) {
            throw "The Authenticode signature did not verify for $Path. Check the certificate chain and timestamp."
        }
    }

    # Sign the application binary before NSIS packages it.
    Sign-And-Verify -Path $appBinary

    # Tauri's custom signing command is invoked for the NSIS uninstaller and the
    # final installer. The static Store config is augmented with the resolved
    # signtool path and certificate thumbprint for this build only.
    $signCommand = @{
        cmd = $signtool.FullName
        args = @(
            "sign", "/s", "My", "/sha1", $certificate.Thumbprint,
            "/fd", "SHA256", "/tr", "http://timestamp.digicert.com",
            "/td", "SHA256", "%1"
        )
    }
    $storeConfig = Get-Content "src-tauri/tauri.microsoftstore.conf.json" -Raw | ConvertFrom-Json
    $storeConfig.bundle.windows | Add-Member -NotePropertyName signCommand -NotePropertyValue $signCommand -Force
    $runtimeConfigPath = Join-Path ([System.IO.Path]::GetTempPath()) ("tauri-store-config-" + [guid]::NewGuid().ToString("N") + ".json")
    $storeConfig | ConvertTo-Json -Depth 10 | Set-Content -Path $runtimeConfigPath -Encoding UTF8

    Invoke-Checked "Create the offline NSIS installer and sign its uninstaller and setup executable" {
        npm run tauri bundle -- --config $runtimeConfigPath
    }

    $installerDirectory = Join-Path $repoRoot "src-tauri/target/release/bundle/nsis"
    $installers = @(Get-ChildItem $installerDirectory -Filter "*.exe" -File -ErrorAction SilentlyContinue)
    if ($installers.Count -ne 1) {
        throw "Expected exactly one new NSIS installer; found $($installers.Count)."
    }

    # Tauri has already signed the NSIS uninstaller and outer installer using
    # bundle.windows.signCommand. Verify the outer signature before publishing.
    & $signtool.FullName verify /pa /v $installers[0].FullName
    if ($LASTEXITCODE -ne 0) {
        throw "The Authenticode signature did not verify for the NSIS installer."
    }

    foreach ($signedFile in @($appBinary, $installers[0].FullName)) {
        $signature = Get-AuthenticodeSignature -FilePath $signedFile
        if ($signature.Status -ne "Valid") {
            throw "Windows reports a non-valid signature for $signedFile: $($signature.StatusMessage)"
        }
    }

    # Microsoft Store requires every PE file embedded in the installer to be
    # signed. Tauri/NSIS can leave stock NSIS plugins unsigned even when the
    # outer installer is signed. Inspect the extracted payload before publishing.
    $sevenZip = Get-Command "7z.exe" -ErrorAction SilentlyContinue
    if ($null -eq $sevenZip) {
        $sevenZipCandidates = @(
            (Join-Path $env:ProgramFiles "7-Zip\7z.exe"),
            (Join-Path \${env:ProgramFiles(x86)} "7-Zip\7z.exe")
        )
        foreach ($candidate in $sevenZipCandidates) {
            if (Test-Path $candidate) {
                $sevenZip = Get-Item $candidate
                break
            }
        }
    }
    if ($null -eq $sevenZip) {
        throw "7-Zip is required to inspect the NSIS payload. Install 7-Zip and ensure 7z.exe is available."
    }

    function Test-PortableExecutable {
        param([Parameter(Mandatory = $true)][string]$Path)

        $stream = [System.IO.File]::OpenRead($Path)
        $reader = $null
        try {
            if ($stream.Length -lt 64) { return $false }
            $reader = [System.IO.BinaryReader]::new($stream)
            if ($reader.ReadUInt16() -ne 0x5A4D) { return $false }
            $stream.Position = 0x3c
            $peOffset = $reader.ReadInt32()
            if ($peOffset -lt 64 -or $peOffset -gt ($stream.Length - 4)) { return $false }
            $stream.Position = $peOffset
            return $reader.ReadUInt32() -eq 0x00004550
        }
        catch {
            return $false
        }
        finally {
            if ($null -ne $reader) {
                $reader.Dispose()
            } else {
                $stream.Dispose()
            }
        }
    }

    $storeExtractDir = Join-Path ([System.IO.Path]::GetTempPath()) ("widget-website-store-payload-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $storeExtractDir -Force | Out-Null
    Write-Host "Extracting installer payload to inspect all embedded PE files..."
    & $sevenZip.Source x -y "-o$storeExtractDir" $installers[0].FullName
    if ($LASTEXITCODE -ne 0) {
        throw "7-Zip could not extract the NSIS installer payload (exit code $LASTEXITCODE)."
    }

    $embeddedPeFiles = @(
        Get-ChildItem $storeExtractDir -Recurse -File |
        Where-Object { Test-PortableExecutable -Path $_.FullName }
    )
    if ($embeddedPeFiles.Count -eq 0) {
        throw "No embedded PE files were discovered in the extracted NSIS installer. Refusing to treat the package as validated."
    }

    $unsignedPeFiles = @()
    foreach ($peFile in $embeddedPeFiles) {
        $signature = Get-AuthenticodeSignature -FilePath $peFile.FullName
        if ($signature.Status -ne "Valid") {
            $unsignedPeFiles += [pscustomobject]@{
                Path = $peFile.FullName.Substring($storeExtractDir.Length).TrimStart('\')
                Status = $signature.Status.ToString()
                Message = $signature.StatusMessage
            }
        }
    }
    if ($unsignedPeFiles.Count -gt 0) {
        $details = ($unsignedPeFiles | ForEach-Object { "$($_.Path) [$($_.Status)]" }) -join "; "
        throw "The NSIS package contains PE files without a valid Authenticode signature. This fails Microsoft Store requirements: $details"
    }

    $installerHash = (Get-FileHash -FilePath $installers[0].FullName -Algorithm SHA256).Hash
    $manifestLines = @(
        "Widget Website Microsoft Store package",
        "Version: $($tauriConfig.version)",
        "Installer: $($installers[0].Name)",
        "SHA256: $installerHash",
        "Embedded PE files verified: $($embeddedPeFiles.Count)",
        "",
        "Embedded PE files:"
    )
    foreach ($peFile in $embeddedPeFiles) {
        $signature = Get-AuthenticodeSignature -FilePath $peFile.FullName
        $relative = $peFile.FullName.Substring($storeExtractDir.Length).TrimStart('\')
        $manifestLines += "$relative | $($signature.SignerCertificate.Subject)"
    }
    $manifestPath = Join-Path $installerDirectory "SHA256SUMS.txt"
    $manifestLines | Set-Content -Path $manifestPath -Encoding UTF8

    Write-Host ""
    Write-Host "Microsoft Store installer created and signed:" -ForegroundColor Green
    Write-Host $installers[0].FullName
    Write-Host "Silent install parameter for Partner Center: /S"
    Write-Warning "This script does not upload the installer or submit the product. Test silent install, Add/Remove Programs, updates and Store package validation before submission."
}
finally {
    if ($null -ne $pfxPath) {
        Remove-Item $pfxPath -Force -ErrorAction SilentlyContinue
    }
    if ($null -ne $runtimeConfigPath) {
        Remove-Item $runtimeConfigPath -Force -ErrorAction SilentlyContinue
    }
    if ($null -ne $storeExtractDir) {
        Remove-Item $storeExtractDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    foreach ($thumbprint in $importedThumbprints) {
        Remove-Item ("Cert:\CurrentUser\My\" + $thumbprint) -Force -ErrorAction SilentlyContinue
    }
    Remove-Item Env:WINDOWS_PFX_BASE64 -ErrorAction SilentlyContinue
    Remove-Item Env:WINDOWS_PFX_PASSWORD -ErrorAction SilentlyContinue
    Pop-Location
}
