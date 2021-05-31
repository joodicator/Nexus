use untitled::derive_crosscast;

fn main() {
}

trait Text { fn str(&self) -> &str; }
trait Message { fn text(&self) -> &dyn Text; }

struct PlainText { text: String }
impl Text for PlainText { fn str(&self) -> &str { &*self.text } }
derive_crosscast!(PlainText, base_traits(Text));

struct TextMessage { text: PlainText }
impl Message for TextMessage { fn text(&self) -> &dyn Text { &self.text } }

