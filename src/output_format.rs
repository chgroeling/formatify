pub enum OutputFormat {
    None,
    LeftAlign(u32),
    LeftAlignTrunc(u32),
    RightAlign(u32),
    RightAlignTrunc(u32),
    RightAlignLTrunc(u32),
}
