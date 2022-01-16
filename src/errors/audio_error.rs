
#[cfg(feature="bass_audio")]
use bass_rs::prelude::BassError;
#[cfg(feature="neb_audio")]
use symphonia::core::errors::Error as SymphoniaError;

#[derive(Debug)]
pub enum AudioError {
    #[cfg(feature="neb_audio")]
    SymphoniaError(SymphoniaError),
    #[cfg(feature="bass_audio")]
    BassError(BassError),


    FileDoesntExist,
    DifferentSong,
}


#[cfg(feature="neb_audio")]
impl From<SymphoniaError> for AudioError {
    fn from(e: SymphoniaError) -> Self {AudioError::SymphoniaError(e)}
}

#[cfg(feature="bass_audio")]
impl From<BassError> for AudioError {
    fn from(e: BassError) -> Self {AudioError::BassError(e)}
}