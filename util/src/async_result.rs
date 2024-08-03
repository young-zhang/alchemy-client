pub type AsyncResult<T> = anyhow::Result<T, anyhow::Error>;
// TODO: migrate to https://github.com/eyre-rs/eyre since Alloy uses it
// also see: https://github.com/eyre-rs/eyre/tree/master/color-eyre
