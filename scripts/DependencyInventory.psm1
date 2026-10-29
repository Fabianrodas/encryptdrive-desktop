#requires -Version 7.0

Set-StrictMode -Version Latest

function Get-Sha256Text {
    param([Parameter(Mandatory = $true)] [string] $Text)
    $bytes = [Text.Encoding]::UTF8.GetBytes($Text)
    $hash = [Security.Cryptography.SHA256]::HashData($bytes)
    return [Convert]::ToHexString($hash)
}

function Read-CargoLockPackageRecords {
    param([Parameter(Mandatory = $true)] [string] $Path)

    $records = [Collections.Generic.List[object]]::new()
    $current = $null
    foreach ($line in Get-Content -LiteralPath $Path) {
        if ($line -ceq '[[package]]') {
            if ($null -ne $current) { $records.Add([pscustomobject]$current) }
            $current = [ordered]@{ name = $null; version = $null; source = $null; checksum = $null }
            continue
        }
        if ($null -eq $current) { continue }
        if ($line -match '^name = "([^"]+)"$') { $current.name = $Matches[1]; continue }
        if ($line -match '^version = "([^"]+)"$') { $current.version = $Matches[1]; continue }
        if ($line -match '^source = "([^"]+)"$') { $current.source = $Matches[1]; continue }
        if ($line -match '^checksum = "([0-9a-fA-F]{64})"$') { $current.checksum = $Matches[1].ToUpperInvariant() }
    }
    if ($null -ne $current) { $records.Add([pscustomobject]$current) }

    foreach ($record in $records) {
        if ([string]::IsNullOrWhiteSpace([string]$record.name) -or [string]::IsNullOrWhiteSpace([string]$record.version)) {
            throw 'Cargo.lock contains a package entry without name or version.'
        }
    }
    return $records.ToArray()
}

function Invoke-CargoMetadata {
    param(
        [Parameter(Mandatory = $true)] [string] $RepositoryRoot,
        [Parameter(Mandatory = $true)] [string[]] $Arguments
    )

    $metadataArguments = @('metadata') + $Arguments
    Push-Location -LiteralPath $RepositoryRoot
    try {
        $output = & cargo @metadataArguments 2>$null | Out-String
        $cargoExitCode = $LASTEXITCODE
    }
    finally { Pop-Location }
    if ($cargoExitCode -ne 0) { throw "cargo metadata failed with exit code $cargoExitCode." }
    try { return $output | ConvertFrom-Json }
    catch { throw "cargo metadata returned invalid JSON: $($_.Exception.Message)" }
}

function Get-ResolvedPackageRecords {
    param(
        [Parameter(Mandatory = $true)] $Metadata,
        [Parameter(Mandatory = $true)] [hashtable] $StableIdByCargoId,
        [Parameter(Mandatory = $true)] [string] $RootCargoId,
        [Parameter(Mandatory = $true)] [string] $ConfigurationName
    )

    $nodesById = @{}
    foreach ($node in @($Metadata.resolve.nodes)) { $nodesById[[string]$node.id] = $node }
    if (-not $nodesById.ContainsKey($RootCargoId)) { throw "Cargo metadata for '$ConfigurationName' is missing the application root node." }

    $edgesById = @{}
    foreach ($node in @($Metadata.resolve.nodes)) {
        $nodeEdges = [Collections.Generic.List[object]]::new()
        foreach ($dependency in @($node.deps)) {
            $kinds = [Collections.Generic.List[string]]::new()
            $conditions = [Collections.Generic.List[string]]::new()
            foreach ($kindRecord in @($dependency.dep_kinds)) {
                $kind = if ([string]::IsNullOrWhiteSpace([string]$kindRecord.kind)) { 'normal' } else { [string]$kindRecord.kind }
                if ($kind -eq 'dev') { continue }
                if (-not $kinds.Contains($kind)) { $kinds.Add($kind) }
                $condition = if ($null -eq $kindRecord.target) { '<unconditional>' } else { [string]$kindRecord.target }
                if (-not $conditions.Contains($condition)) { $conditions.Add($condition) }
            }
            if ($kinds.Count -eq 0) { continue }
            $childId = [string]$dependency.pkg
            if (-not $StableIdByCargoId.ContainsKey($childId)) { throw "Cargo dependency '$childId' in '$ConfigurationName' is not present in the locked package set." }
            $nodeEdges.Add([pscustomobject]@{
                    package_id = [string]$StableIdByCargoId[$childId]
                    cargo_package_id = $childId
                    dependency_name = [string]$dependency.name
                    kinds = @($kinds | Sort-Object -Unique)
                    target_conditions = @($conditions | Sort-Object -Unique)
                })
        }
        $edgesById[[string]$node.id] = @($nodeEdges | Sort-Object package_id, dependency_name)
    }

    $depthById = @{$RootCargoId = 0}
    $pending = [Collections.Generic.Queue[string]]::new()
    $pending.Enqueue($RootCargoId)
    while ($pending.Count -gt 0) {
        $currentId = $pending.Dequeue()
        $nextDepth = [int]$depthById[$currentId] + 1
        foreach ($edge in @($edgesById[$currentId])) {
            $childId = [string]$edge.cargo_package_id
            if (-not $nodesById.ContainsKey($childId)) { throw "Cargo dependency node '$childId' is missing from '$ConfigurationName' resolution." }
            if (-not $depthById.ContainsKey($childId) -or $nextDepth -lt [int]$depthById[$childId]) {
                $depthById[$childId] = $nextDepth
                $pending.Enqueue($childId)
            }
        }
    }

    $result = [Collections.Generic.List[object]]::new()
    foreach ($cargoId in @($depthById.Keys | Sort-Object { $StableIdByCargoId[[string]$_] })) {
        $node = $nodesById[[string]$cargoId]
        $depth = [int]$depthById[$cargoId]
        $relationship = if ($depth -eq 0) { 'root' } elseif ($depth -eq 1) { 'direct' } else { 'transitive' }
        $result.Add([pscustomobject]@{
                package_id = [string]$StableIdByCargoId[[string]$cargoId]
                dependency_depth = $depth
                relationship = $relationship
                enabled_features = @($node.features | ForEach-Object { [string]$_ } | Sort-Object -Unique)
                dependencies = @($edgesById[[string]$cargoId] | ForEach-Object {
                        [pscustomobject]@{
                            package_id = $_.package_id
                            dependency_name = $_.dependency_name
                            kinds = $_.kinds
                            target_conditions = $_.target_conditions
                        }
                    })
            })
    }

    return $result.ToArray()
}

function Find-DumpbinPath {
    $command = Get-Command dumpbin.exe -ErrorAction SilentlyContinue
    if ($null -ne $command) { return $command.Source }

    $vswhereCandidates = [Collections.Generic.List[string]]::new()
    if (-not [string]::IsNullOrWhiteSpace(${env:ProgramFiles(x86)})) {
        $vswhereCandidates.Add((Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'))
    }
    if (-not [string]::IsNullOrWhiteSpace($env:ProgramFiles)) {
        $vswhereCandidates.Add((Join-Path $env:ProgramFiles 'Microsoft Visual Studio\Installer\vswhere.exe'))
    }
    foreach ($vswhere in $vswhereCandidates) {
        if (-not (Test-Path -LiteralPath $vswhere -PathType Leaf)) { continue }
        $installation = (& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null | Select-Object -First 1)
        if ([string]::IsNullOrWhiteSpace([string]$installation)) { continue }
        $toolRoot = Join-Path $installation 'VC\Tools\MSVC'
        if (-not (Test-Path -LiteralPath $toolRoot -PathType Container)) { continue }
        foreach ($version in @(Get-ChildItem -LiteralPath $toolRoot -Directory | Sort-Object Name -Descending)) {
            $candidate = Join-Path $version.FullName 'bin\Hostx64\x64\dumpbin.exe'
            if (Test-Path -LiteralPath $candidate -PathType Leaf) { return $candidate }
        }
    }
    return $null
}

function Get-PeImportedDllNames {
    param([Parameter(Mandatory = $true)] [string] $DumpbinPath, [Parameter(Mandatory = $true)] [string] $ExecutablePath)
    $output = & $DumpbinPath /DEPENDENTS $ExecutablePath 2>&1 | ForEach-Object { [string]$_ }
    if ($LASTEXITCODE -ne 0) { throw "dumpbin failed for executable '$([IO.Path]::GetFileName($ExecutablePath))' with exit code $LASTEXITCODE." }
    $names = @($output | ForEach-Object {
            if ($_ -match '^\s+([A-Za-z0-9_.+-]+\.dll)\s*$') { $Matches[1].ToLowerInvariant() }
        } | Sort-Object -Unique)
    if ($names.Count -eq 0) { throw "dumpbin did not report any PE import DLLs for '$([IO.Path]::GetFileName($ExecutablePath))'." }
    return $names
}

function Get-BuildLogObservation {
    param(
        [Parameter(Mandatory = $true)] [string] $LogPath,
        [Parameter(Mandatory = $true)] [hashtable] $StableIdByCargoId,
        [Parameter(Mandatory = $true)] $Packages
    )
    if (-not (Test-Path -LiteralPath $LogPath -PathType Leaf)) {
        return [pscustomobject]@{ status = 'unavailable'; sha256 = $null; compiled_package_observations = @() }
    }

    $observed = @{}
    foreach ($line in Get-Content -LiteralPath $LogPath) {
        if ($line -match '^\s*Compiling\s+([A-Za-z0-9_.+-]+)\s+v([^\s]+)') {
            $name = $Matches[1]
            $version = $Matches[2]
            $candidates = @($Packages | Where-Object { $_.name -ceq $name -and $_.version -ceq $version })
            if ($candidates.Count -eq 0) { throw "Build log '$([IO.Path]::GetFileName($LogPath))' contains package '$name v$version' outside Cargo.lock." }
            foreach ($candidate in $candidates) { $observed[[string]$StableIdByCargoId[[string]$candidate.id]] = $true }
        }
    }
    $logHash = (Get-FileHash -LiteralPath $LogPath -Algorithm SHA256).Hash.ToUpperInvariant()
    return [pscustomobject]@{
        status = 'observed-compiling-lines-only'
        sha256 = $logHash
        compiled_package_observations = @($observed.Keys | Sort-Object)
    }
}

function Get-VerifiedBuildEvidence {
    param(
        [Parameter(Mandatory = $true)] [string] $ManifestPath,
        [Parameter(Mandatory = $true)] [string] $RepositoryRoot,
        [Parameter(Mandatory = $true)] [hashtable] $StableIdByCargoId,
        [Parameter(Mandatory = $true)] $Packages
    )
    Import-Module (Join-Path $PSScriptRoot 'BuildArtifacts.psm1') -Force
    $verification = Test-BuildArtifactManifest -ManifestPath $ManifestPath -RepositoryRoot $RepositoryRoot
    $manifest = Get-Content -LiteralPath $ManifestPath -Raw | ConvertFrom-Json
    $manifestHash = [string]$manifest.environment.cargo_lock_sha256
    $actualLockHash = (Get-FileHash -LiteralPath (Join-Path $RepositoryRoot 'Cargo.lock') -Algorithm SHA256).Hash.ToUpperInvariant()
    if (-not $manifestHash.Equals($actualLockHash, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Build manifest Cargo.lock SHA-256 does not match the current lockfile.'
    }

    $dumpbinPath = Find-DumpbinPath
    $variants = [Collections.Generic.List[object]]::new()
    foreach ($artifact in @($manifest.artifacts | Sort-Object variant)) {
        $executablePath = Join-Path $RepositoryRoot ([string]$artifact.relative_path).Replace('/', [IO.Path]::DirectorySeparatorChar)
        $logPath = Join-Path (Split-Path -Parent $executablePath) 'cargo-build.log'
        $logObservation = Get-BuildLogObservation -LogPath $logPath -StableIdByCargoId $StableIdByCargoId -Packages $Packages
        $imports = if ($null -ne $dumpbinPath) { @(Get-PeImportedDllNames -DumpbinPath $dumpbinPath -ExecutablePath $executablePath) } else { @() }
        $variants.Add([pscustomobject]@{
                name = [string]$artifact.variant
                profile = [string]$artifact.profile
                target = [string]$artifact.target
                requested_features = @($artifact.features | ForEach-Object { [string]$_ } | Sort-Object -Unique)
                executable_size_bytes = [long]$artifact.size_bytes
                executable_sha256 = ([string]$artifact.sha256).ToUpperInvariant()
                compiled_package_observations = $logObservation.compiled_package_observations
                build_log_observation_status = $logObservation.status
                build_log_sha256 = $logObservation.sha256
                static_pe_imports = $imports
            })
    }

    if ($variants.Count -ne 2) { throw 'Verified build manifest did not provide both required variants.' }
    return [pscustomobject]@{
        run_id = [string]$verification.run_id
        source_revision = [string]$verification.source_revision
        source_dirty = [bool]$verification.source_dirty
        cargo_version = [string]$manifest.environment.cargo_version
        rustc_version = [string]$manifest.environment.rustc_version
        rustc_commit_hash = [string]$manifest.environment.rustc_commit_hash
        target = [string]$manifest.environment.target
        cargo_lock_sha256 = $actualLockHash
        native_import_status = if ($null -ne $dumpbinPath) { 'collected' } else { 'blocked-dumpbin-unavailable' }
        native_import_tool = if ($null -ne $dumpbinPath) { 'dumpbin /DEPENDENTS' } else { $null }
        variants = $variants.ToArray()
    }
}

function New-DependencyInventory {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)] [string] $RepositoryRoot,
        [string] $ManifestPath
    )

    $root = (Resolve-Path -LiteralPath $RepositoryRoot).Path
    $lockPath = Join-Path $root 'Cargo.lock'
    if (-not (Test-Path -LiteralPath $lockPath -PathType Leaf)) { throw 'Cargo.lock is missing.' }
    $lockRecords = @(Read-CargoLockPackageRecords -Path $lockPath)
    $fullMetadata = Invoke-CargoMetadata -RepositoryRoot $root -Arguments @('--locked', '--format-version', '1')
    if (@($fullMetadata.packages).Count -ne $lockRecords.Count) {
        throw "Cargo metadata package count ($(@($fullMetadata.packages).Count)) does not match Cargo.lock entry count ($($lockRecords.Count))."
    }

    $workspaceIds = @{}
    foreach ($id in @($fullMetadata.workspace_members)) { $workspaceIds[[string]$id] = $true }
    $packageByLockIdentity = @{}
    foreach ($package in @($fullMetadata.packages)) {
        $source = if ($null -eq $package.source) { '' } else { [string]$package.source }
        $identity = "$($package.name)|$($package.version)|$source"
        if ($packageByLockIdentity.ContainsKey($identity)) { throw "Duplicate Cargo metadata identity '$($package.name) $($package.version)'." }
        $packageByLockIdentity[$identity] = $package
    }

    $lockedPackages = [Collections.Generic.List[object]]::new()
    $stableIdByCargoId = @{}
    foreach ($lockRecord in $lockRecords) {
        $source = if ($null -eq $lockRecord.source) { '' } else { [string]$lockRecord.source }
        $identity = "$($lockRecord.name)|$($lockRecord.version)|$source"
        if (-not $packageByLockIdentity.ContainsKey($identity)) {
            throw "Cargo.lock package '$($lockRecord.name) $($lockRecord.version)' is missing from cargo metadata."
        }
        $package = $packageByLockIdentity[$identity]
        $cargoId = [string]$package.id
        $isWorkspace = $workspaceIds.ContainsKey($cargoId)
        $sourceKind = if ($isWorkspace) { 'workspace' }
        elseif ($package.source -like 'registry+*') { 'registry' }
        elseif ($package.source -like 'git+*') { 'git' }
        else { 'local-path' }
        $sourceIdentity = if ($isWorkspace) { $null }
        elseif ([string]::IsNullOrWhiteSpace([string]$package.source)) { [string]$package.id }
        else { [string]$package.source }
        $sourceFingerprint = if ($null -eq $sourceIdentity) { $null } else { Get-Sha256Text -Text $sourceIdentity }
        $packageId = if ($isWorkspace) { "workspace:$($package.name)@$($package.version)" }
        else { "$sourceKind`:$($package.name)@$($package.version):$sourceFingerprint" }
        if ($stableIdByCargoId.ContainsKey($cargoId)) { throw "Duplicate Cargo package identifier '$cargoId'." }
        $stableIdByCargoId[$cargoId] = $packageId
        $hasLicense = -not [string]::IsNullOrWhiteSpace([string]$package.license)
        $hasLicenseFile = -not [string]::IsNullOrWhiteSpace([string]$package.license_file)
        $licenseFileValue = $null
        $licenseFileValueRedacted = $false
        if ($hasLicenseFile) {
            $rawLicenseFile = [string]$package.license_file
            if ([IO.Path]::IsPathRooted($rawLicenseFile) -or $rawLicenseFile -match '^(?:[A-Za-z]:[\\/]|[\\/]{2}|/)') {
                $licenseFileValueRedacted = $true
            }
            else { $licenseFileValue = $rawLicenseFile }
        }
        $licenseStatus = if ($hasLicense -and $hasLicenseFile) { 'license-expression-and-license-file-declared' }
        elseif ($hasLicense) { 'license-expression-declared' }
        elseif ($hasLicenseFile) { 'license-file-declared' }
        else { 'license-metadata-missing' }
        $lockedPackages.Add([pscustomobject]@{
                id = $packageId
                name = [string]$package.name
                version = [string]$package.version
                source_kind = $sourceKind
                source_fingerprint = $sourceFingerprint
                checksum = if ($null -eq $lockRecord.checksum) { $null } else { [string]$lockRecord.checksum }
                license_expression = if ($hasLicense) { [string]$package.license } else { $null }
                license_file_declared = [bool]$hasLicenseFile
                license_file_value = $licenseFileValue
                license_file_value_redacted = [bool]$licenseFileValueRedacted
                license_metadata_status = $licenseStatus
                workspace_member = [bool]$isWorkspace
            })
    }
    $rootPackage = @($fullMetadata.packages | Where-Object { $_.name -ceq 'encryptdrive-desktop' -and $workspaceIds.ContainsKey([string]$_.id) })
    if ($rootPackage.Count -ne 1) { throw "Expected one workspace package named 'encryptdrive-desktop'; found $($rootPackage.Count)." }
    $rootCargoId = [string]$rootPackage[0].id

    $configs = [Collections.Generic.List[object]]::new()
    foreach ($configuration in @(@{ name = 'shell'; features = @() }, @{ name = 'design-preview'; features = @('design-preview') })) {
        $args = @('--locked', '--format-version', '1', '--filter-platform', 'x86_64-pc-windows-msvc')
        if (@($configuration.features).Count -gt 0) { $args += @('--features', 'encryptdrive-desktop/design-preview') }
        $metadata = Invoke-CargoMetadata -RepositoryRoot $root -Arguments $args
        $resolved = @(Get-ResolvedPackageRecords -Metadata $metadata -StableIdByCargoId $stableIdByCargoId -RootCargoId $rootCargoId -ConfigurationName $configuration.name)
        $configs.Add([pscustomobject]@{
                name = $configuration.name
                requested_features = @($configuration.features)
                root_package_id = [string]$stableIdByCargoId[$rootCargoId]
                resolved_package_count = $resolved.Count
                direct_dependency_count = @($resolved | Where-Object { $_.relationship -eq 'direct' }).Count
                transitive_dependency_count = @($resolved | Where-Object { $_.relationship -eq 'transitive' }).Count
                resolved_packages = $resolved
            })
    }

    $buildEvidence = if (-not [string]::IsNullOrWhiteSpace($ManifestPath)) {
        Get-VerifiedBuildEvidence -ManifestPath $ManifestPath -RepositoryRoot $root -StableIdByCargoId $stableIdByCargoId -Packages $fullMetadata.packages
    }
    else { $null }

    return [pscustomobject]@{
        schema_version = 1
        target = 'x86_64-pc-windows-msvc'
        dependency_resolution = 'cargo metadata --locked; no manifest or lockfile mutation'
        lockfile = [pscustomobject]@{
            sha256 = (Get-FileHash -LiteralPath $lockPath -Algorithm SHA256).Hash.ToUpperInvariant()
            package_count = $lockedPackages.Count
            packages = @($lockedPackages | Sort-Object id)
        }
        configurations = $configs.ToArray()
        build_evidence = $buildEvidence
    }
}

function Test-DependencyInventoryDocument {
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)] $Inventory)

    if ($null -eq $Inventory.PSObject.Properties['schema_version']) { throw "Inventory is missing required field 'schema_version'." }
    if ([int]$Inventory.schema_version -ne 1) { throw "Unsupported inventory schema_version '$($Inventory.schema_version)'." }
    if ([string]$Inventory.target -cne 'x86_64-pc-windows-msvc') { throw "Inventory target must be 'x86_64-pc-windows-msvc'." }
    if ($null -eq $Inventory.PSObject.Properties['lockfile'] -or $null -eq $Inventory.lockfile) { throw "Inventory is missing required object 'lockfile'." }
    if ([string]$Inventory.lockfile.sha256 -notmatch '^[0-9A-Fa-f]{64}$') { throw "lockfile.sha256 must be a 64-character SHA-256 value." }
    if ($null -eq $Inventory.lockfile.PSObject.Properties['packages']) { throw "Inventory is missing required field 'lockfile.packages'." }
    $locked = @($Inventory.lockfile.packages)
    if ([int]$Inventory.lockfile.package_count -ne $locked.Count) { throw 'lockfile.package_count does not match lockfile.packages.' }
    $lockedById = @{}
    foreach ($package in $locked) {
        if ([string]::IsNullOrWhiteSpace([string]$package.id) -or [string]$package.id -match '[/\\]|://' -or [string]$package.id -match '^[A-Za-z]:') {
            throw 'Each locked package must have a path-free stable id.'
        }
        if ($lockedById.ContainsKey([string]$package.id)) { throw "Duplicate locked package identity '$($package.id)'." }
        if ([string]$package.source_kind -notin @('workspace', 'registry', 'git', 'local-path')) { throw "Unsupported source_kind for package '$($package.id)'." }
        if ($package.source_kind -eq 'workspace') {
            if ($null -ne $package.source_fingerprint) { throw "Workspace package '$($package.id)' must not have a source fingerprint." }
        }
        elseif ([string]$package.source_fingerprint -notmatch '^[0-9A-Fa-f]{64}$') {
            throw "Non-workspace package '$($package.id)' requires a source fingerprint."
        }
        $expectedPackageId = if ($package.source_kind -eq 'workspace') { "workspace:$($package.name)@$($package.version)" }
        else { "$($package.source_kind):$($package.name)@$($package.version):$($package.source_fingerprint)" }
        if ([string]$package.id -cne $expectedPackageId) { throw "Package '$($package.id)' stable id does not match its identity fields." }
        if ($null -ne $package.checksum -and [string]$package.checksum -notmatch '^[0-9A-Fa-f]{64}$') { throw "Package '$($package.id)' has an invalid Cargo.lock checksum." }
        if ($package.source_kind -eq 'registry' -and $null -eq $package.checksum) { throw "Registry package '$($package.id)' requires a Cargo.lock checksum." }
        if ($package.workspace_member -isnot [bool]) { throw "Package '$($package.id)' workspace_member must be a boolean." }
        if ([bool]$package.workspace_member -ne ($package.source_kind -eq 'workspace')) { throw "Package '$($package.id)' workspace_member does not match source_kind." }
        if ($package.license_expression -isnot [string] -and $null -ne $package.license_expression) { throw "Package '$($package.id)' license_expression must preserve the raw string or null." }
        if ($package.license_file_declared -isnot [bool]) { throw "Package '$($package.id)' license_file_declared must be a boolean." }
        if ($package.license_file_value_redacted -isnot [bool]) { throw "Package '$($package.id)' license_file_value_redacted must be a boolean." }
        if ($package.license_file_declared) {
            if ($package.license_file_value_redacted) {
                if ($null -ne $package.license_file_value) { throw "Package '$($package.id)' redacted license_file metadata must not retain its path value." }
            }
            elseif ($package.license_file_value -isnot [string] -or [IO.Path]::IsPathRooted([string]$package.license_file_value) -or [string]$package.license_file_value -match '^(?:[A-Za-z]:[\\/]|[\\/]{2}|/)') {
                throw "Package '$($package.id)' license_file metadata must preserve a relative path or redact an absolute path."
            }
        }
        elseif ($null -ne $package.license_file_value -or $package.license_file_value_redacted) {
            throw "Package '$($package.id)' license_file value is present without a declaration."
        }
        $hasLicenseExpression = -not [string]::IsNullOrWhiteSpace([string]$package.license_expression)
        $expectedLicenseStatus = if ($hasLicenseExpression -and $package.license_file_declared) { 'license-expression-and-license-file-declared' }
        elseif ($hasLicenseExpression) { 'license-expression-declared' }
        elseif ($package.license_file_declared) { 'license-file-declared' }
        else { 'license-metadata-missing' }
        if ([string]$package.license_metadata_status -cne $expectedLicenseStatus) {
            throw "Package '$($package.id)' license_metadata_status does not match its recorded Cargo metadata fields."
        }
        $lockedById[[string]$package.id] = $package
    }

    if ($null -eq $Inventory.PSObject.Properties['configurations']) { throw "Inventory is missing required field 'configurations'." }
    $configurations = @($Inventory.configurations)
    if ($configurations.Count -ne 2) { throw 'Inventory must contain exactly the shell and design-preview configurations.' }
    $configByName = @{}
    foreach ($configuration in $configurations) {
        $name = [string]$configuration.name
        if ($name -notin @('shell', 'design-preview')) { throw "Unsupported configuration '$name'." }
        if ($configByName.ContainsKey($name)) { throw "Duplicate configuration '$name'." }
        $configByName[$name] = $configuration
        if ([string]::IsNullOrWhiteSpace([string]$configuration.root_package_id)) { throw "Configuration '$name' is missing root_package_id." }
        $expectedFeatures = if ($name -eq 'shell') { @() } else { @('design-preview') }
        if ((@($configuration.requested_features) -join '|') -cne ($expectedFeatures -join '|')) { throw "Configuration '$name' has an unexpected requested feature set." }
        if ($null -eq $configuration.PSObject.Properties['resolved_packages']) { throw "Configuration '$name' is missing resolved_packages." }
        $resolved = @($configuration.resolved_packages)
        if ([int]$configuration.resolved_package_count -ne $resolved.Count) { throw "Configuration '$name' resolved_package_count does not match its package list." }
        $resolvedById = @{}
        foreach ($entry in $resolved) {
            $packageId = [string]$entry.package_id
            if (-not $lockedById.ContainsKey($packageId)) { throw "Resolved package '$packageId' is not present in lockfile." }
            if ($resolvedById.ContainsKey($packageId)) { throw "Configuration '$name' contains duplicate resolved package '$packageId'." }
            if ($entry.dependency_depth -isnot [ValueType] -or [int]$entry.dependency_depth -lt 0) { throw "Configuration '$name' has an invalid dependency_depth." }
            $expectedRelationship = if ([int]$entry.dependency_depth -eq 0) { 'root' } elseif ([int]$entry.dependency_depth -eq 1) { 'direct' } else { 'transitive' }
            if ([string]$entry.relationship -cne $expectedRelationship) { throw "Configuration '$name' relationship does not match dependency_depth for '$packageId'." }
            $resolvedById[$packageId] = $entry
        }
        if (-not $resolvedById.ContainsKey([string]$configuration.root_package_id)) { throw "Configuration '$name' root package is not resolved." }
        $rootEntry = $resolvedById[[string]$configuration.root_package_id]
        if ([int]$rootEntry.dependency_depth -ne 0 -or [string]$rootEntry.relationship -cne 'root') { throw "Configuration '$name' root package must have depth 0 and relationship 'root'." }
        foreach ($entry in $resolved) {
            foreach ($dependency in @($entry.dependencies)) {
                if (-not $resolvedById.ContainsKey([string]$dependency.package_id)) { throw "Configuration '$name' dependency '$($dependency.package_id)' is not in the resolved package set." }
            }
        }
    }
    foreach ($required in @('shell', 'design-preview')) { if (-not $configByName.ContainsKey($required)) { throw "Inventory is missing required configuration '$required'." } }

    if ($null -eq $Inventory.PSObject.Properties['build_evidence']) { throw "Inventory is missing required field 'build_evidence'." }
    if ($null -ne $Inventory.build_evidence) {
        $evidence = $Inventory.build_evidence
        if ([string]$evidence.run_id -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$') { throw 'Build evidence run_id is invalid.' }
        if ([string]$evidence.source_revision -notmatch '^[0-9a-fA-F]{40}$') { throw 'Build evidence source_revision must be a full Git commit ID.' }
        if ($evidence.source_dirty -isnot [bool]) { throw 'Build evidence source_dirty must be a JSON boolean.' }
        foreach ($field in @('cargo_version', 'rustc_version')) { if ([string]::IsNullOrWhiteSpace([string]$evidence.$field)) { throw "Build evidence $field cannot be empty." } }
        if ([string]$evidence.target -cne [string]$Inventory.target) { throw 'Build evidence target does not match inventory target.' }
        if ([string]$evidence.cargo_lock_sha256 -cne [string]$Inventory.lockfile.sha256) { throw 'Build evidence lockfile hash does not match inventory.' }
        $nativeStatus = [string]$evidence.native_import_status
        if ($nativeStatus -notin @('collected', 'blocked-dumpbin-unavailable', 'not-collected')) { throw "Unsupported native import status '$nativeStatus'." }
        $variants = @($evidence.variants)
        if ($variants.Count -ne 2 -or (@($variants | ForEach-Object { [string]$_.name } | Sort-Object) -join '|') -cne 'design-preview|shell') {
            throw 'Build evidence must include shell and design-preview variants.'
        }
        if ($nativeStatus -eq 'collected' -and [string]$evidence.native_import_tool -cne 'dumpbin /DEPENDENTS') { throw 'Collected native imports must identify dumpbin /DEPENDENTS.' }
        if ($nativeStatus -eq 'blocked-dumpbin-unavailable' -and $null -ne $evidence.native_import_tool) { throw 'Blocked native import evidence must not claim a collector.' }
        foreach ($variant in $variants) {
            if ([string]$variant.profile -cne 'release' -or [string]$variant.target -cne [string]$Inventory.target) { throw "Build evidence for '$($variant.name)' has an unexpected profile or target." }
            $expectedFeatures = if ([string]$variant.name -eq 'shell') { @() } else { @('design-preview') }
            if ((@($variant.requested_features) -join '|') -cne ($expectedFeatures -join '|')) { throw "Build evidence for '$($variant.name)' has an unexpected feature set." }
            if ([string]$variant.executable_sha256 -notmatch '^[0-9A-Fa-f]{64}$' -or $variant.executable_size_bytes -isnot [ValueType] -or [long]$variant.executable_size_bytes -le 0) { throw "Build evidence for '$($variant.name)' has invalid executable identity." }
            if ([string]$variant.build_log_observation_status -notin @('observed-compiling-lines-only', 'unavailable')) { throw "Build evidence for '$($variant.name)' has an unsupported build-log status." }
            if ($null -ne $variant.build_log_sha256 -and [string]$variant.build_log_sha256 -notmatch '^[0-9A-Fa-f]{64}$') { throw "Build evidence for '$($variant.name)' has an invalid build-log SHA-256." }
            foreach ($packageId in @($variant.compiled_package_observations)) {
                if (-not $lockedById.ContainsKey([string]$packageId)) { throw "Build log observation '$packageId' is not present in lockfile." }
            }
            $imports = @($variant.static_pe_imports)
            if ($nativeStatus -eq 'collected' -and $imports.Count -eq 0) { throw "Build evidence for '$($variant.name)' has no static PE imports." }
            if ($nativeStatus -ne 'collected' -and $imports.Count -ne 0) { throw "Build evidence for '$($variant.name)' contains imports without a collector." }
            foreach ($dll in $imports) { if ([string]$dll -notmatch '^[A-Za-z0-9_.+-]+\.dll$') { throw "Build evidence for '$($variant.name)' contains an invalid PE import name." } }
            if (@($imports | Sort-Object -Unique).Count -ne $imports.Count) { throw "Build evidence for '$($variant.name)' contains duplicate PE import names." }
        }
        if ($null -eq $evidence.rustc_commit_hash -or [string]$evidence.rustc_commit_hash -notmatch '^[0-9a-fA-F]{40}$') {
            throw 'Build evidence rustc_commit_hash must be a full commit ID.'
        }
    }
    return $true
}

Export-ModuleMember -Function New-DependencyInventory, Test-DependencyInventoryDocument
