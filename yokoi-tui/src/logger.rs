use log::{
    Log, Metadata, Record,
    kv::{self, Key, Value, VisitSource},
};
use std::io::Write;

pub struct Logger<W>(pub W);

impl<W: Send + Sync> Log for Logger<W>
where
    for<'a> &'a W: Write,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with("yokoi")
    }

    fn log(&self, record: &Record) {
        struct Visitor<W>(W);

        impl<'kvs, W: Write> VisitSource<'kvs> for Visitor<W> {
            fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), kv::Error> {
                writeln!(self.0, "{key}: {value}").map_err(Into::into)
            }
        }

        if self.enabled(record.metadata()) {
            if record.args().as_str().map(str::len) != Some(0) {
                writeln!(&self.0, "{}", record.args()).unwrap();
            }
            let mut visitor = Visitor(&self.0);
            record.key_values().visit(&mut visitor).unwrap();
            writeln!(&self.0).unwrap();
        }
    }

    fn flush(&self) {
        (&self.0).flush().unwrap();
    }
}

impl<W: Send + Sync + 'static> Logger<W>
where
    for<'a> &'a W: Write,
{
    pub fn init(self) {
        log::set_logger(Box::leak(Box::new(self))).expect("setting logger");
    }
}
