use log::{
    Log, Metadata, Record,
    kv::{self, Key, Value, VisitSource, VisitValue},
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
        struct Visitor<'kvs, W>(W, Option<Key<'kvs>>);

        impl<'v, 'kvs, W: Write> VisitValue<'v> for Visitor<'kvs, W> {
            fn visit_any(&mut self, value: Value) -> Result<(), kv::Error> {
                writeln!(
                    self.0,
                    "{}: {value}",
                    self.1.as_ref().expect("key from VisitSource")
                )?;
                Ok(())
            }

            fn visit_null(&mut self) -> Result<(), kv::Error> {
                Ok(())
            }
        }

        impl<'kvs, W: Write> VisitSource<'kvs> for Visitor<'kvs, W> {
            fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), kv::Error> {
                self.1 = Some(key);
                value.visit(self)
            }
        }

        if self.enabled(record.metadata()) {
            if record.args().as_str().map(str::len) != Some(0) {
                writeln!(&self.0, "{}", record.args()).unwrap();
            }
            let mut visitor = Visitor(&self.0, None);
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
