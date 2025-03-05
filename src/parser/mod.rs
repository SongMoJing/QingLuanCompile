pub mod toml;

pub enum ReturnValue {
    Success(),
    Error(ReturnErr),
}

pub enum ReturnErr {
	FormatError,
}