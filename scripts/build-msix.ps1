param(
    [string]$PackageIdentityName = $env:MSIX_PACKAGE_IDENTITY_NAME,
    [string]$PackagePublisher = $env:MSIX_PACKAGE_PUBLISHER,
    [string]$PublisherDisplayName = $env:MSIX_PUBLISHER_DISPLAY_NAME
)

$ErrorActionPreference = "Stop"

function Stop-Build([string]$Message) {
    throw "[build-msix] $Message"
}

function Invoke-Checked([string]$Label, [scriptblock]$Command) {
    Write-Host ""
    Write-Host "== $Label ==" -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Stop-Build "$Label falhou (código $LASTEXITCODE)."
    }
}

foreach ($entry in @(
    @{ Name = "PackageIdentityName"; Value = $PackageIdentityName },
    @{ Name = "PackagePublisher"; Value = $PackagePublisher },
    @{ Name = "PublisherDisplayName"; Value = $PublisherDisplayName }
)) {
    if ([string]::IsNullOrWhiteSpace($entry.Value) -or $entry.Value -match "^__.*__$") {
        Stop-Build "Informe $($entry.Name) com o valor exato fornecido pelo Partner Center. Nenhum valor fictício será usado."
    }
}

if ($PackageIdentityName -notmatch "^[A-Za-z0-9.\-]{3,50}$") {
    Stop-Build "PackageIdentityName parece inválido. Copie o campo Name de Package identity no Partner Center."
}

if ([string]::IsNullOrWhiteSpace((Get-Command "winapp" -ErrorAction SilentlyContinue))) {
    Stop-Build "WinApp CLI não encontrada. Instale-a com: winget install microsoft.winappcli --source winget"
}

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot

try {
    $tauriConfig = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
    if ($tauriConfig.productName -ne "Floatly") {
        Stop-Build "O nome de produto no Tauri não é Floatly."
    }

    if ($tauriConfig.version -notmatch "^(\d+)\.(\d+)\.(\d+)$") {
        Stop-Build "A versão Tauri deve usar três componentes numéricos, por exemplo 0.1.0."
    }
    $packageVersion = "$($tauriConfig.version).0"

    Invoke-Checked "Validar configuração MSIX" { npm run check:msix-config }

    Invoke-Checked "Gerar ícones da aplicação" { npm run icons }

    Invoke-Checked "Compilar o executável Tauri sem instalador NSIS/MSI" {
        npm run tauri -- build --no-bundle -- --locked
    }

    $appBinary = Join-Path $repoRoot "src-tauri/target/release/widget-website.exe"
    if (!(Test-Path $appBinary)) {
        Stop-Build "Executável não encontrado: $appBinary"
    }

    $outputRoot = Join-Path $repoRoot "dist/msix"
    if (Test-Path $outputRoot) {
        Remove-Item -Recurse -Force $outputRoot
    }
    $stageDirectory = Join-Path $outputRoot "staging"
    $assetsDirectory = Join-Path $stageDirectory "Assets"
    New-Item -ItemType Directory -Path $assetsDirectory -Force | Out-Null

    Copy-Item $appBinary (Join-Path $stageDirectory "widget-website.exe") -Force

    $manifestSource = Join-Path $repoRoot "packaging/msix/Package.appxmanifest.template.xml"
    $manifestPath = Join-Path $stageDirectory "Package.appxmanifest"
    Copy-Item $manifestSource $manifestPath -Force

    [xml]$manifest = Get-Content $manifestPath -Raw
    $identity = $manifest.SelectSingleNode("/*[local-name()='Package']/*[local-name()='Identity']")
    $displayName = $manifest.SelectSingleNode("/*[local-name()='Package']/*[local-name()='Properties']/*[local-name()='PublisherDisplayName']")
    if ($null -eq $identity -or $null -eq $displayName) {
        Stop-Build "O manifesto MSIX não contém Identity ou PublisherDisplayName."
    }

    $identity.SetAttribute("Name", $PackageIdentityName)
    $identity.SetAttribute("Publisher", $PackagePublisher)
    $identity.SetAttribute("Version", $packageVersion)
    $displayName.InnerText = $PublisherDisplayName
    $manifest.Save($manifestPath)

    $logoPath = Join-Path $repoRoot "icons/widget.svg"
    Invoke-Checked "Gerar imagens de pacote a partir do ícone Floatly" {
        winapp manifest update-assets $logoPath --manifest $manifestPath
    }

    $outputFileName = "Floatly_$($packageVersion)_x64.msix"
    $outputPath = Join-Path $outputRoot $outputFileName
    if (Test-Path $outputPath) { Remove-Item $outputPath -Force }

    Invoke-Checked "Empacotar MSIX sem assinatura para submissão à Microsoft Store" {
        winapp pack $stageDirectory --no-sign --output $outputPath
    }

    if (!(Test-Path $outputPath)) {
        Stop-Build "O comando terminou sem criar o pacote esperado: $outputPath"
    }

    $hash = (Get-FileHash $outputPath -Algorithm SHA256).Hash
    @(
        "Floatly MSIX package",
        "Version: $packageVersion",
        "Architecture: x64",
        "Package identity: $PackageIdentityName",
        "SHA256: $hash",
        "File: $outputFileName"
    ) | Set-Content -Path (Join-Path $outputRoot "SHA256SUMS.txt") -Encoding UTF8

    Write-Host ""
    Write-Host "Pacote MSIX pronto para upload no Partner Center:" -ForegroundColor Green
    Write-Host $outputPath
    Write-Host "SHA-256: $hash"
    Write-Warning "O pacote foi criado sem assinatura. Esse é o fluxo esperado para envio à Microsoft Store; a Store assina o pacote após a certificação."
}
finally {
    Pop-Location
}
