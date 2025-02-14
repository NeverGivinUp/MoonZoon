use super::class_id_generator;
use crate::*;
use std::iter;

// ------ ------
//   Element
// ------ ------

pub struct RawSvgEl<DomElement: Into<web_sys::SvgElement>> {
    class_id: ClassId,
    dom_builder: DomBuilder<DomElement>,
}

impl RawSvgEl<web_sys::SvgElement> {
    pub fn new(tag: &str) -> Self {
        <Self as RawEl>::new(tag)
    }
}

impl<DomElement: Into<web_sys::SvgElement> + Clone + JsCast> RawSvgEl<DomElement> {
    pub fn dom_element_type<T: Into<web_sys::SvgElement> + JsCast>(self) -> RawSvgEl<T> {
        let element = self.dom_builder.__internal_element().unchecked_into::<T>();
        let dom_builder = DomBuilder::new(element).__internal_transfer_callbacks(self.dom_builder);
        RawSvgEl {
            class_id: self.class_id,
            dom_builder,
        }
    }
}

impl<DomElement: Into<web_sys::SvgElement> + Clone + JsCast> From<RawSvgEl<DomElement>>
    for RawElement
{
    fn from(raw_svg_el: RawSvgEl<DomElement>) -> Self {
        RawElement::SvgEl(raw_svg_el.dom_element_type::<web_sys::SvgElement>())
    }
}

impl<DomElement: Into<web_sys::SvgElement> + Into<web_sys::Node>> IntoDom for RawSvgEl<DomElement> {
    fn into_dom(self) -> Dom {
        self.dom_builder.into_dom()
    }
}

impl<DomElement: Into<web_sys::SvgElement> + Clone + JsCast> Element for RawSvgEl<DomElement> {
    fn into_raw_element(self) -> RawElement {
        RawElement::SvgEl(self.dom_element_type::<web_sys::SvgElement>())
    }
}

impl<DomElement: Into<web_sys::SvgElement>> IntoIterator for RawSvgEl<DomElement> {
    type Item = Self;
    type IntoIter = iter::Once<Self>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        iter::once(self)
    }
}

// ------ ------
//     RawEl
// ------ ------

impl<DomElement> RawEl for RawSvgEl<DomElement>
// Warning: "Global" bounds with `JsValue` or `JsCast` or `AsRef<web_sys::SvgElement>` break Rust Analyzer.
where
    DomElement: AsRef<web_sys::Node>
        + Into<web_sys::EventTarget>
        + AsRef<web_sys::EventTarget>
        + AsRef<web_sys::Element>
        + Into<web_sys::Element>
        + Into<web_sys::SvgElement>
        + Clone
        + 'static,
{
    type DomElement = DomElement;

    fn new(tag: &str) -> Self
    where
        DomElement: JsCast,
    {
        let class_id = class_id_generator().next_class_id();

        let mut dom_builder = DomBuilder::new_svg(tag);
        dom_builder = class_id.map(move |class_id| dom_builder.class(class_id.unwrap_throw()));

        Self {
            class_id: class_id.clone(),
            dom_builder: dom_builder
                .after_removed(move |_| class_id_generator().remove_class_id(class_id)),
        }
    }

    fn update_dom_builder(
        mut self,
        updater: impl FnOnce(DomBuilder<Self::DomElement>) -> DomBuilder<Self::DomElement>,
    ) -> Self {
        self.dom_builder = updater(self.dom_builder);
        self
    }

    fn dom_element(&self) -> Self::DomElement {
        self.dom_builder.__internal_element()
    }

    fn class_id(&self) -> ClassId {
        self.class_id.clone()
    }

    fn from_dom_element(dom_element: Self::DomElement) -> Self {
        let mut dom_builder = DomBuilder::new(dom_element);

        let class_id = class_id_generator().next_class_id();
        dom_builder = class_id.map(move |class_id| dom_builder.class(class_id.unwrap_throw()));

        Self {
            class_id: class_id.clone(),
            dom_builder: dom_builder
                .after_removed(move |_| class_id_generator().remove_class_id(class_id)),
        }
    }

    fn focus(self) -> Self
    where
        Self::DomElement: AsRef<web_sys::HtmlElement>,
    {
        unimplemented!();
    }

    fn focus_signal(self, _focus: impl Signal<Item = bool> + Unpin + 'static) -> Self
    where
        Self::DomElement: AsRef<web_sys::HtmlElement>,
    {
        unimplemented!();
    }
}
