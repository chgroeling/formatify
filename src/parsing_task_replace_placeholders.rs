use super::output_format::OutputFormat;
use super::parsing_context::ParsingContext;
use super::parsing_task::ParsingTask;
use super::peek_char_iterator::PeekCharIterator;

use std::collections::HashMap;

pub struct ParsingTaskReplacePlaceholders;

impl ParsingTask for ParsingTaskReplacePlaceholders {
    type Item = char;
    type Output = String;

    /// Called in case the context should be initialized
    fn init<'a>(
        inp: &'a str,
        key_value: &'a HashMap<&'a str, String>,
    ) -> ParsingContext<'a, Self::Item> {
        let vec: Vec<_> = inp.chars().collect();
        ParsingContext::<'_, Self::Item> {
            key_value,
            iter: PeekCharIterator::new(vec),
            vout: Vec::<char>::new(),
            format: OutputFormat::None,
        }
    }

    fn error(context: &mut ParsingContext<'_, Self::Item>) {
        context.vout.extend(context.iter.get_mark2cur().unwrap());
    }

    fn process_char(context: &mut ParsingContext<'_, Self::Item>, ch: char) {
        context.vout.push(ch);
    }

    fn process_char_placeholder(context: &mut ParsingContext<'_, Self::Item>, ch: char) {
        context.vout.push(ch);
    }

    fn process_str_placeholder(context: &mut ParsingContext<'_, Self::Item>, arg: String) {
        let Some(repl_str) = context.key_value.get(arg.as_str()) else {
            Self::error(context);
            return;
        };
        let repl = repl_str.chars();
        match context.format {
            OutputFormat::None => {
                context.vout.extend(repl);
            }

            OutputFormat::LeftAlign(la) => {
                context.vout.extend(repl.clone());
                let value_len = repl.into_iter().count();
                let len_diff = (la as i32) - (value_len as i32);
                if len_diff > 0 {
                    for _i in 0..len_diff {
                        context.vout.push(' ');
                    }
                }
            }

            OutputFormat::LeftAlignTrunc(la) => {
                let value_len = repl.clone().count();
                let len_diff = (la as i32) - (value_len as i32);

                match len_diff {
                    _ if len_diff > 0 => {
                        context.vout.extend(repl);
                        for _i in 0..len_diff {
                            context.vout.push(' ');
                        }
                    }

                    _ if len_diff < 0 => {
                        let let_cmp = (value_len as i32) + len_diff - 1;
                        for (idx, ch) in repl.into_iter().enumerate() {
                            if idx >= let_cmp as usize {
                                break;
                            }
                            context.vout.push(ch);
                        }
                        context.vout.push('…');
                    }
                    _ => {
                        // len_diff ==0
                        context.vout.extend(repl);
                    }
                }
            }

            OutputFormat::RightAlign(ra) => {
                let value_len = repl.clone().into_iter().count();
                let len_diff = (ra as i32) - (value_len as i32);
                if len_diff > 0 {
                    for _i in 0..len_diff {
                        context.vout.push(' ');
                    }
                }
                context.vout.extend(repl);
            }

            OutputFormat::RightAlignTrunc(ra) => {
                let value_len = repl.clone().count();
                let len_diff = (ra as i32) - (value_len as i32);

                match len_diff {
                    _ if len_diff > 0 => {
                        for _i in 0..len_diff {
                            context.vout.push(' ');
                        }
                        context.vout.extend(repl);
                    }

                    _ if len_diff < 0 => {
                        let let_cmp = (value_len as i32) + len_diff - 1;
                        for (idx, ch) in repl.into_iter().enumerate() {
                            if idx >= let_cmp as usize {
                                break;
                            }
                            context.vout.push(ch);
                        }
                        context.vout.push('…');
                    }
                    _ => {
                        // len_diff ==0
                        context.vout.extend(repl);
                    }
                }
            }
        }
    }

    fn done(context: ParsingContext<'_, Self::Item>) -> Self::Output {
        context.vout.into_iter().collect()
    }
}
