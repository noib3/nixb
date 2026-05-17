use core::ffi::CStr;

/// TODO: docs.
pub enum StoreParam<'a> {
    /// TODO: docs.
    AddressingStyle(&'a str),
    /// TODO: docs.
    Base64SshPublicHostKey(&'a str),
    /// TODO: docs.
    BufferSize(&'a str),
    /// TODO: docs.
    BuildDir(&'a str),
    /// TODO: docs.
    CheckMount(&'a str),
    /// TODO: docs.
    Compress(&'a str),
    /// TODO: docs.
    Compression(&'a str),
    /// TODO: docs.
    CompressionLevel(&'a str),
    /// TODO: docs.
    Endpoint(&'a str),
    /// TODO: docs.
    IgnoreGcDeleteFailure(&'a str),
    /// TODO: docs.
    IndexDebugInfo(&'a str),
    /// TODO: docs.
    LocalNarCache(&'a str),
    /// TODO: docs.
    Log(&'a str),
    /// TODO: docs.
    LogCompression(&'a str),
    /// TODO: docs.
    LogFd(&'a str),
    /// TODO: docs.
    LowerStore(&'a str),
    /// TODO: docs.
    LsCompression(&'a str),
    /// TODO: docs.
    MaxConnectionAge(&'a str),
    /// TODO: docs.
    MaxConnections(&'a str),
    /// TODO: docs.
    MultipartChunkSize(&'a str),
    /// TODO: docs.
    MultipartThreshold(&'a str),
    /// TODO: docs.
    MultipartUpload(&'a str),
    /// TODO: docs.
    NarinfoCompression(&'a str),
    /// TODO: docs.
    ParallelCompression(&'a str),
    /// TODO: docs.
    PathInfoCacheSize(&'a str),
    /// TODO: docs.
    Priority(&'a str),
    /// TODO: docs.
    Profile(&'a str),
    /// TODO: docs.
    ReadOnly(&'a str),
    /// TODO: docs.
    Real(&'a str),
    /// TODO: docs.
    Region(&'a str),
    /// TODO: docs.
    RemoteProgram(&'a str),
    /// TODO: docs.
    RemoteStore(&'a str),
    /// TODO: docs.
    RemountHook(&'a str),
    /// TODO: docs.
    RequireSigs(&'a str),
    /// TODO: docs.
    RetryAttempts(&'a str),
    /// TODO: docs.
    RetryDelay(&'a str),
    /// TODO: docs.
    RetryDelayRateLimited(&'a str),
    /// TODO: docs.
    RetryMaxDelay(&'a str),
    /// TODO: docs.
    Root(&'a str),
    /// TODO: docs.
    Scheme(&'a str),
    /// TODO: docs.
    SecretKey(&'a str),
    /// TODO: docs.
    SecretKeys(&'a str),
    /// TODO: docs.
    SshKey(&'a str),
    /// TODO: docs.
    State(&'a str),
    /// TODO: docs.
    StorageClass(&'a str),
    /// TODO: docs.
    Store(&'a str),
    /// TODO: docs.
    SystemFeatures(&'a str),
    /// TODO: docs.
    TlsCertificate(&'a str),
    /// TODO: docs.
    TlsPrivateKey(&'a str),
    /// TODO: docs.
    Trusted(&'a str),
    /// TODO: docs.
    UpperLayer(&'a str),
    /// TODO: docs.
    UseRootsDaemon(&'a str),
    /// TODO: docs.
    WantMassQuery(&'a str),
    /// TODO: docs.
    WriteNarListing(&'a str),
}

impl<'a> StoreParam<'a> {
    pub(crate) const fn key(&self) -> &'static CStr {
        match self {
            Self::AddressingStyle(_) => c"addressing-style",
            Self::Base64SshPublicHostKey(_) => c"base64-ssh-public-host-key",
            Self::BufferSize(_) => c"buffer-size",
            Self::BuildDir(_) => c"build-dir",
            Self::CheckMount(_) => c"check-mount",
            Self::Compress(_) => c"compress",
            Self::Compression(_) => c"compression",
            Self::CompressionLevel(_) => c"compression-level",
            Self::Endpoint(_) => c"endpoint",
            Self::IgnoreGcDeleteFailure(_) => c"ignore-gc-delete-failure",
            Self::IndexDebugInfo(_) => c"index-debug-info",
            Self::LocalNarCache(_) => c"local-nar-cache",
            Self::Log(_) => c"log",
            Self::LogCompression(_) => c"log-compression",
            Self::LogFd(_) => c"log-fd",
            Self::LowerStore(_) => c"lower-store",
            Self::LsCompression(_) => c"ls-compression",
            Self::MaxConnectionAge(_) => c"max-connection-age",
            Self::MaxConnections(_) => c"max-connections",
            Self::MultipartChunkSize(_) => c"multipart-chunk-size",
            Self::MultipartThreshold(_) => c"multipart-threshold",
            Self::MultipartUpload(_) => c"multipart-upload",
            Self::NarinfoCompression(_) => c"narinfo-compression",
            Self::ParallelCompression(_) => c"parallel-compression",
            Self::PathInfoCacheSize(_) => c"path-info-cache-size",
            Self::Priority(_) => c"priority",
            Self::Profile(_) => c"profile",
            Self::ReadOnly(_) => c"read-only",
            Self::Real(_) => c"real",
            Self::Region(_) => c"region",
            Self::RemoteProgram(_) => c"remote-program",
            Self::RemoteStore(_) => c"remote-store",
            Self::RemountHook(_) => c"remount-hook",
            Self::RequireSigs(_) => c"require-sigs",
            Self::RetryAttempts(_) => c"retry-attempts",
            Self::RetryDelay(_) => c"retry-delay",
            Self::RetryDelayRateLimited(_) => c"retry-delay-rate-limited",
            Self::RetryMaxDelay(_) => c"retry-max-delay",
            Self::Root(_) => c"root",
            Self::Scheme(_) => c"scheme",
            Self::SecretKey(_) => c"secret-key",
            Self::SecretKeys(_) => c"secret-keys",
            Self::SshKey(_) => c"ssh-key",
            Self::State(_) => c"state",
            Self::StorageClass(_) => c"storage-class",
            Self::Store(_) => c"store",
            Self::SystemFeatures(_) => c"system-features",
            Self::TlsCertificate(_) => c"tls-certificate",
            Self::TlsPrivateKey(_) => c"tls-private-key",
            Self::Trusted(_) => c"trusted",
            Self::UpperLayer(_) => c"upper-layer",
            Self::UseRootsDaemon(_) => c"use-roots-daemon",
            Self::WantMassQuery(_) => c"want-mass-query",
            Self::WriteNarListing(_) => c"write-nar-listing",
        }
    }

    pub(crate) const fn value(&self) -> &'a str {
        match self {
            Self::AddressingStyle(value)
            | Self::Base64SshPublicHostKey(value)
            | Self::BufferSize(value)
            | Self::BuildDir(value)
            | Self::CheckMount(value)
            | Self::Compress(value)
            | Self::Compression(value)
            | Self::CompressionLevel(value)
            | Self::Endpoint(value)
            | Self::IgnoreGcDeleteFailure(value)
            | Self::IndexDebugInfo(value)
            | Self::LocalNarCache(value)
            | Self::Log(value)
            | Self::LogCompression(value)
            | Self::LogFd(value)
            | Self::LowerStore(value)
            | Self::LsCompression(value)
            | Self::MaxConnectionAge(value)
            | Self::MaxConnections(value)
            | Self::MultipartChunkSize(value)
            | Self::MultipartThreshold(value)
            | Self::MultipartUpload(value)
            | Self::NarinfoCompression(value)
            | Self::ParallelCompression(value)
            | Self::PathInfoCacheSize(value)
            | Self::Priority(value)
            | Self::Profile(value)
            | Self::ReadOnly(value)
            | Self::Real(value)
            | Self::Region(value)
            | Self::RemoteProgram(value)
            | Self::RemoteStore(value)
            | Self::RemountHook(value)
            | Self::RequireSigs(value)
            | Self::RetryAttempts(value)
            | Self::RetryDelay(value)
            | Self::RetryDelayRateLimited(value)
            | Self::RetryMaxDelay(value)
            | Self::Root(value)
            | Self::Scheme(value)
            | Self::SecretKey(value)
            | Self::SecretKeys(value)
            | Self::SshKey(value)
            | Self::State(value)
            | Self::StorageClass(value)
            | Self::Store(value)
            | Self::SystemFeatures(value)
            | Self::TlsCertificate(value)
            | Self::TlsPrivateKey(value)
            | Self::Trusted(value)
            | Self::UpperLayer(value)
            | Self::UseRootsDaemon(value)
            | Self::WantMassQuery(value)
            | Self::WriteNarListing(value) => value,
        }
    }
}
