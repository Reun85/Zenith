#![allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerId {
    id: u32,
}

trait Layer
where
    Self: crate::debug::Debug,
{
    fn set_id(&mut self, id: LayerId);
    fn get_id(&self) -> LayerId;
}

type DynLayerType = std::rc::Rc<dyn Layer>;
#[derive(Debug)]
struct LayerStack {
    layers: Vec<DynLayerType>,
    first_overlay: usize,
}
impl LayerStack {
    fn new() -> Self {
        Self {
            layers: Vec::new(),
            first_overlay: 0,
        }
    }
    fn push_layer(&mut self, layer: impl Into<DynLayerType>) {
        self.layers.insert(self.first_overlay, layer.into());
        self.first_overlay += 1;
    }
    fn push_overlay(&mut self, overlay: DynLayerType) {
        self.layers.push(overlay);
    }
    fn pop_layer(&mut self, layer: impl Layer) {
        let index = self
            .layers
            .iter()
            .position(|l| l.get_id() == layer.get_id())
            .unwrap();
        self.layers.remove(index);
        self.first_overlay -= 1;
    }
    fn pop_overlay(&mut self, overlay: impl Layer) {
        let index = self
            .layers
            .iter()
            .position(|l| l.get_id() == overlay.get_id())
            .unwrap();
        self.layers.remove(index);
    }
}
