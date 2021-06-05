#[macro_use]
extern crate untitled;

fn main() {
}

trait Text { fn str(&self) -> &str; }
trait Message { fn text(&self) -> &dyn Text; }

struct PlainText { text: String }
impl Text for PlainText { fn str(&self) -> &str { &*self.text } }
DynCast!(PlainText, base_traits(Text));

struct TextMessage { text: PlainText }
impl Message for TextMessage { fn text(&self) -> &dyn Text { &self.text } }
DynCast!(TextMessage, base_traits(Message));
