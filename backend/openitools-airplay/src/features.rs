use airplay::config::Features;

pub const FEATURES: Features = {
    type Fe = Features;
    Fe::Photo
        // taken from UxPlay
        .union(Fe::VideoFairPlay)
        .union(Fe::Slideshow)
        .union(Fe::Unknown6)
        .union(Fe::ScreenMirroring)
        .union(Fe::AirPlayAudio)
        .union(Fe::Unknown10)
        .union(Fe::AudioRedundant)
        .union(Fe::FPSAPv2p5_AES_GCM)
        .union(Fe::MFiHardware)
        .union(Fe::MFiSoft_FairPlay)
        .union(Fe::AudioMetaCovers)
        .union(Fe::AudioMetaProgress)
        .union(Fe::AudioMetaTxtDAAP)
        .union(Fe::ReceiveAudioPCM)
        .union(Fe::ReceiveAudioALAC)
        .union(Fe::ReceiveAudioAAC_LC)
        .union(Fe::Unknown21)
        .union(Fe::AudioUnencrypted)
        .union(Fe::iTunes4WEncryption)
        .union(Fe::Unknown28)
        .union(Fe::UnifiedAdvertisingInfo)
        // from the default of airplay crate
        .union(Fe::LegacyPairing)
        .union(Fe::BufferedAudio)
        .union(Fe::NTPClock)
        .union(Fe::PTPClock)
};
