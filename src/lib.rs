use chrono::{DateTime, FixedOffset, Local, Offset, Utc};
use env_logger::fmt::{style, Formatter};
use env_logger::{TimestampPrecision};
use log::Record;
use std::fmt::Display;
use std::io;
use std::io::Write;
use std::time::SystemTime;

const DATETIME_FMT_SECOND: &str = "%Y-%m-%d %H:%M:%S %:z";
const DATETIME_FMT_3F: &str = "%Y-%m-%d %H:%M:%S%.3f %:z";
const DATETIME_FMT_6F: &str = "%Y-%m-%d %H:%M:%S%.6f %:z";

#[derive(Clone, Debug)]
pub struct TimeZoneFormatEnv {
    pub datetime_fmt: &'static str,
    pub offset: FixedOffset,
    pub module_path: bool,
    pub target: bool,
    pub level: bool,
    pub indent: Option<usize>,
    pub suffix: &'static str,
}

impl Default for TimeZoneFormatEnv {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl TimeZoneFormatEnv {
    pub fn new(offset_value: Option<i32>, timestamp_precision: Option<TimestampPrecision>) -> Self {
        let offset = if let Some(offset_value) = offset_value {
            FixedOffset::east_opt(offset_value).unwrap_or(Local::now().offset().fix())
        } else {
            Local::now().offset().fix()
        };
        let datetime_fmt = if let Some(p) = timestamp_precision {
            match p {
                TimestampPrecision::Seconds => DATETIME_FMT_SECOND,
                TimestampPrecision::Millis => DATETIME_FMT_3F,
                TimestampPrecision::Micros => DATETIME_FMT_6F,
                TimestampPrecision::Nanos => DATETIME_FMT_6F,
            }
        } else {
            DATETIME_FMT_3F
        };
        Self {
            datetime_fmt,
            offset,
            module_path: false,
            target: true,
            level: true,
            indent: Some(4),
            suffix: "\n",
        }
    }
}

//type SubtleStyle = StyledValue<&'static str>;
struct StyledValue<T> {
    style: style::Style,
    value: T,
}

impl<T: Display> Display for StyledValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let style = self.style;
        write!(f, "{style}")?;
        self.value.fmt(f)?;
        write!(f, "{style:#}")?;
        Ok(())
    }
}

pub struct TimeZoneFormat<'a> {
    env: &'a TimeZoneFormatEnv,
    buf: &'a mut Formatter,
    written_header_value: bool,
}

impl<'a> TimeZoneFormat<'a> {
    pub fn new(buf: &'a mut Formatter, env: &'a TimeZoneFormatEnv) -> Self {
        Self {
            env,
            buf,
            written_header_value: false,
        }
    }
    pub fn write(mut self, record: &Record) -> io::Result<()> {
        self.write_timestamp()?;
        self.write_level(record)?;
        self.write_module_path(record)?;
        self.write_target(record)?;
        self.finish_header()?;

        self.write_args(record)?;
        write!(self.buf, "{}", self.env.suffix)
    }

    fn subtle_style(&self, text: &'static str) -> &'static str {
        text
    }

    fn write_header_value<T>(&mut self, value: T) -> io::Result<()>
    where
        T: Display,
    {
        if !self.written_header_value {
            self.written_header_value = true;

            let open_brace = self.subtle_style("[");
            write!(self.buf, "{}{}", open_brace, value)
        } else {
            write!(self.buf, " {}", value)
        }
    }

    fn write_level(&mut self, record: &Record) -> io::Result<()> {
        if !self.env.level {
            return Ok(());
        }

        let level = {
            let level = record.level();
            StyledValue {
                style: self.buf.default_level_style(level),
                value: level,
            }
        };

        self.write_header_value(format_args!("{:<5}", level))
    }

    fn write_timestamp(&mut self) -> io::Result<()> {
        let datetime_str = DateTime::<Utc>::from(SystemTime::now())
            .with_timezone(&self.env.offset)
            .format(self.env.datetime_fmt);
        self.write_header_value(datetime_str)
    }

    fn write_module_path(&mut self, record: &Record) -> io::Result<()> {
        if !self.env.module_path {
            return Ok(());
        }

        if let Some(module_path) = record.module_path() {
            self.write_header_value(module_path)
        } else {
            Ok(())
        }
    }

    fn write_target(&mut self, record: &Record) -> io::Result<()> {
        if !self.env.target {
            return Ok(());
        }

        match record.target() {
            "" => Ok(()),
            target => self.write_header_value(target),
        }
    }

    fn finish_header(&mut self) -> io::Result<()> {
        if self.written_header_value {
            let close_brace = self.subtle_style("]");
            write!(self.buf, "{} ", close_brace)
        } else {
            Ok(())
        }
    }

    fn write_args(&mut self, record: &Record) -> io::Result<()> {
        match self.env.indent {
            // Fast path for no indentation
            None => write!(self.buf, "{}", record.args()),

            Some(indent_count) => {
                // Create a wrapper around the buffer only if we have to actually indent the message

                struct IndentWrapper<'a, 'b: 'a> {
                    fmt: &'a mut TimeZoneFormat<'b>,
                    indent_count: usize,
                }

                impl<'a, 'b> Write for IndentWrapper<'a, 'b> {
                    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                        let mut first = true;
                        for chunk in buf.split(|&x| x == b'\n') {
                            if !first {
                                write!(
                                    self.fmt.buf,
                                    "{}{:width$}",
                                    self.fmt.env.suffix,
                                    "",
                                    width = self.indent_count
                                )?;
                            }
                            self.fmt.buf.write_all(chunk)?;
                            first = false;
                        }

                        Ok(buf.len())
                    }

                    fn flush(&mut self) -> io::Result<()> {
                        self.fmt.buf.flush()
                    }
                }

                // The explicit scope here is just to make older versions of Rust happy
                {
                    let mut wrapper = IndentWrapper {
                        fmt: self,
                        indent_count,
                    };
                    write!(wrapper, "{}", record.args())?;
                }

                Ok(())
            }
        }
    }
}
