use crate::*;
use once_cell::race::OnceBox;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
    iter, mem,
    sync::Arc,
};
use web_sys::{CssStyleDeclaration, CssStyleRule, CssStyleSheet, HtmlStyleElement};

pub mod named_color;

mod align;
pub use align::Align;

mod background;
pub use background::Background;

mod borders;
pub use borders::{Border, Borders};

mod clip;
pub use clip::Clip;

mod cursor;
pub use cursor::{Cursor, CursorIcon};

mod font;
pub use font::{Font, FontFamily, FontLine, FontWeight};

mod height;
pub use height::Height;

mod layer_index;
pub use layer_index::LayerIndex;

mod padding;
pub use padding::Padding;

mod rounded_corners;
pub use rounded_corners::{IntoOptionRadius, Radius, RoundedCorners};

mod scrollbars;
pub use scrollbars::Scrollbars;

mod shadows;
pub use shadows::{Shadow, Shadows};

mod spacing;
pub use spacing::Spacing;

mod transitions;
pub use transitions::{Transition, Transitions};

mod transform;
pub use transform::Transform;

mod visible;
pub use visible::Visible;

mod width;
pub use width::Width;

// --

pub type U32Width = u32;
pub type U32Height = u32;

pub struct CssPropValue<'a> {
    pub(crate) value: Cow<'a, str>,
    pub(crate) important: bool,
}

// ------ StaticCSSProps ------
/// Css properties to be added to the generated html.
#[derive(Default)]
pub struct StaticCSSProps<'a>(BTreeMap<&'a str, CssPropValue<'a>>);

impl<'a> StaticCSSProps<'a> {
    pub fn insert(&mut self, name: &'a str, value: impl IntoCowStr<'a>) {
        self.0.insert(
            name,
            CssPropValue {
                value: value.into_cow_str(),
                important: false,
            },
        );
    }

    pub fn insert_important(&mut self, name: &'a str, value: impl IntoCowStr<'a>) {
        self.0.insert(
            name,
            CssPropValue {
                value: value.into_cow_str(),
                important: true,
            },
        );
    }

    pub fn remove(&mut self, name: &'a str) -> Option<CssPropValue> {
        self.0.remove(name)
    }
}

impl<'a> IntoIterator for StaticCSSProps<'a> {
    type Item = (&'a str, CssPropValue<'a>);
    type IntoIter = std::collections::btree_map::IntoIter<&'a str, CssPropValue<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> Extend<(&'a str, CssPropValue<'a>)> for StaticCSSProps<'a> {
    fn extend<T: IntoIterator<Item = (&'a str, CssPropValue<'a>)>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

// ------ DynamicCSSProps ------

pub type DynamicCSSProps = BTreeMap<Cow<'static, str>, BoxedCssSignal>;

// ------ BoxedCssSignal ------

pub type BoxedCssSignal = Box<dyn Signal<Item = Box<dyn IntoOptionCowStr<'static>>> + Unpin>;

// @TODO replace with a new function? https://github.com/Pauan/rust-signals/blob/master/CHANGELOG.md#0322---2021-06-13
pub fn box_css_signal(
    signal: impl Signal<Item = impl IntoOptionCowStr<'static> + 'static> + Unpin + 'static,
) -> BoxedCssSignal {
    Box::new(signal.map(|value| Box::new(value) as Box<dyn IntoOptionCowStr<'static>>))
}

// ------ StaticCSSClasses ------

pub type StaticCSSClasses<'a> = BTreeSet<&'a str>;

// ------ DynamicCSSClasses ------

pub type DynamicCSSClasses = BTreeMap<Cow<'static, str>, Box<dyn Signal<Item = bool> + Unpin>>;

// ------ units ------

pub fn px<'a>(px: impl IntoCowStr<'a>) -> Cow<'a, str> {
    [&px.into_cow_str(), "px"].concat().into()
}

pub fn ch<'a>(ch: impl IntoCowStr<'a>) -> Cow<'a, str> {
    [&ch.into_cow_str(), "ch"].concat().into()
}

// ------ Style ------

/// Trait to be implemented to enable the use for styling.
/// Every `struct` such as [Align] and [Background] needs to implement
/// this trait so they can be used by [Styleable] implementations with
/// the `s()` method within a `Zoon` element.
pub trait Style<'a>: Default {
    fn new() -> Self {
        Self::default()
    }

    fn merge_with_group(self, group: StyleGroup<'a>) -> StyleGroup<'a>;
}

// ------ StyleGroup ------

/// Css styles that can be added on a raw html element or globally with a
/// selector.
#[derive(Default)]
pub struct StyleGroup<'a> {
    /// The `css selector` where the styles apply.
    pub selector: Cow<'a, str>,
    pub static_css_props: StaticCSSProps<'a>,
    pub dynamic_css_props: DynamicCSSProps,
    // --- not applicable to global styles (only directly to elements) ---
    pub static_css_classes: StaticCSSClasses<'a>,
    pub dynamic_css_classes: DynamicCSSClasses,
    pub resize_handlers: Vec<Arc<dyn Fn(U32Width, U32Height)>>,
}

impl<'a> StyleGroup<'a> {
    /// Create a set of css properties for the specific selector.
    /// More information about `css selectors` at <https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Selectors>.
    /// # Example
    /// We can use already existing classes to apply different styles.
    /// ```no_run
    /// use zoon::*;
    /// let button = Button::new()
    ///     .update_raw_el(|raw_el| {
    ///         raw_el.style_group(
    ///             StyleGroup::new(".button")
    ///                 .style("background", "purple")
    ///                 .style("padding", "10px"),
    ///         )
    ///     })
    ///     .label("Click me");
    /// ```
    /// # Example
    /// Here we add a transition to animate the button when the user hovers it.
    /// ```no_run
    /// use zoon::*;
    /// let button = Button::new()
    ///     .update_raw_el(|raw_el| {
    ///         raw_el.style_group(StyleGroup::new(":hover").style("margin-right", "40%"))
    ///     })
    ///     .s(Transitions::new([
    ///         Transition::property("margin-right").duration(2000)
    ///     ]))
    ///     .label("Hover me");
    /// ```
    pub fn new(selector: impl IntoCowStr<'a>) -> Self {
        Self {
            selector: selector.into_cow_str(),
            ..Default::default()
        }
    }

    /// Add a css a property to a specific selector with a `key` and `value`.
    /// # Example
    /// ```no_run
    /// use zoon::*;
    /// use zoon::RawEl;
    ///
    ///  let button = Button::new()
    ///         .update_raw_el(|el| el.style_group(StyleGroup::new(":hover").style("background", "purple")))
    ///         .label("Click me");
    pub fn style(mut self, name: &'a str, value: impl Into<Cow<'a, str>>) -> Self {
        self.static_css_props.insert(name, value.into());
        self
    }

    /// Add a css property to a specific selector followed by the `!important`
    /// rule. This example shows how to add the rule event if
    /// it is not necessary in this specific case for displaying the correct
    /// background.
    /// # Example
    /// ```no_run
    /// use zoon::*;
    /// use zoon::RawEl;
    ///
    ///  let button = Button::new()
    ///         .update_raw_el(|el| el.style_group(StyleGroup::new(".button")
    /// .style_important("background", "purple")))
    ///         .label("Click me");
    pub fn style_important(mut self, name: &'a str, value: impl Into<Cow<'a, str>>) -> Self {
        self.static_css_props.insert_important(name, value.into());
        self
    }

    /// Update the group style depending of the signal's state.
    /// ```no_run
    /// use zoon::*;
    ///
    /// let (is_hovered, hover_signal) = Mutable::new_and_signal(false);
    /// let button = Button::new()
    ///     .update_raw_el(|el| {
    ///         el.style_group(
    ///             StyleGroup::new(".button")
    ///                 .style_signal("background", hover_signal.map_bool(|| "pink", || "purple")),
    ///         )
    ///     })
    ///     .on_hovered_change(move |hover| is_hovered.set(hover))
    ///     .label("Hover me");
    /// ```
    pub fn style_signal(
        mut self,
        name: impl IntoCowStr<'static>,
        value: impl Signal<Item = impl IntoOptionCowStr<'static> + 'static> + Unpin + 'static,
    ) -> Self {
        self.dynamic_css_props
            .insert(name.into_cow_str(), box_css_signal(value));
        self
    }

    pub fn class(mut self, class: &'a str) -> Self {
        self.static_css_classes.insert(class);
        self
    }

    pub fn class_signal(
        mut self,
        class: impl IntoCowStr<'static>,
        enabled: impl Signal<Item = bool> + Unpin + 'static,
    ) -> Self {
        self.dynamic_css_classes
            .insert(class.into_cow_str(), Box::new(enabled));
        self
    }

    pub fn on_resize(
        mut self,
        handler: impl FnOnce(U32Width, U32Height) + Clone + 'static,
    ) -> Self {
        self.resize_handlers.push(Arc::new(move |width, height| {
            handler.clone()(width, height)
        }));
        self
    }
}

// ------ StyleGroupHandle ------

pub struct StyleGroupHandle {
    rule_id: u32,
    _task_handles: Vec<TaskHandle>,
}

impl Drop for StyleGroupHandle {
    fn drop(&mut self) {
        global_styles().remove_rule(self.rule_id);
    }
}

// ------ global_styles ------

/// Set styles that are globally used in your application.
/// Very convenient for customizing the design at one single place.
/// # Example
/// How to style every button in your app in few lines. The [Button] element has
/// the `button` class attached by default to it.
/// ```no_run
/// use zoon::{named_color::*, *};
///
/// global_styles().style_group(
///     StyleGroup::new(".button")
///         .style("background", "purple")
///         .style("padding", "10px"),
/// );
/// let button = Button::new().label("Click me");
/// ```
/// # Example
/// Global styles can be used to add regular css classes.
/// ```no_run
/// use zoon::{named_color::*, *};
///
/// global_styles().style_group(
///     StyleGroup::new(".my_button_class")
///         .style("background", "purple")
///         .style("padding", "10px"),
/// );
/// let button = Button::new()
///     .label("Click me")
///     .update_raw_el(|el| el.class("my_button_class"));
/// ```
pub fn global_styles() -> &'static GlobalStyles {
    static GLOBAL_STYLES: OnceBox<GlobalStyles> = OnceBox::new();
    GLOBAL_STYLES.get_or_init(|| Box::new(GlobalStyles::new()))
}

pub struct GlobalStyles {
    sheet: SendWrapper<CssStyleSheet>,
    rule_ids: MonotonicIds,
}

impl GlobalStyles {
    fn new() -> Self {
        let style_element: HtmlStyleElement = document()
            .create_element("style")
            .expect_throw("style: create_element failed")
            .unchecked_into();
        document()
            .head()
            .expect_throw("style: head failed")
            .append_child(&style_element)
            .expect_throw("style: append_child failed");
        let sheet = style_element
            .sheet()
            .expect_throw("style: sheet failed")
            .unchecked_into();
        Self {
            sheet: SendWrapper::new(sheet),
            rule_ids: MonotonicIds::default(),
        }
    }

    pub fn style_group(&self, group: StyleGroup) -> &Self {
        let (_, task_handles) = self.style_group_inner(group, false);
        mem::forget(task_handles);
        self
    }

    #[must_use]
    pub fn style_group_droppable(&self, group: StyleGroup) -> StyleGroupHandle {
        let (rule_id, _task_handles) = self.style_group_inner(group, true);
        StyleGroupHandle {
            rule_id,
            _task_handles,
        }
    }

    // --

    fn style_group_inner(&self, group: StyleGroup, droppable: bool) -> (u32, Vec<TaskHandle>) {
        let (rule_id_and_index, ids_lock) = self.rule_ids.add_new_id();
        let empty_rule = [&group.selector, "{}"].concat();

        self.sheet
            .insert_rule_with_index(&empty_rule, rule_id_and_index)
            .unwrap_or_else(|_| {
                panic!("invalid CSS selector: `{}`", &group.selector);
            });

        let declaration = self
            .sheet
            .css_rules()
            .expect_throw("failed to get global CSS rules")
            .item(rule_id_and_index)
            .expect_throw("failed to get selected global CSS rule")
            .unchecked_into::<CssStyleRule>()
            .style();

        drop(ids_lock);

        for (name, css_prop_value) in group.static_css_props {
            set_css_property(
                &declaration,
                name,
                &css_prop_value.value,
                css_prop_value.important,
            );
        }

        let declaration = Arc::new(SendWrapper::new(declaration));
        let mut task_handles = Vec::new();
        for (name, value_signal) in group.dynamic_css_props {
            let declaration = Arc::clone(&declaration);
            let task = value_signal.for_each_sync(move |value| {
                if let Some(value) = value.into_option_cow_str() {
                    // @TODO allow to set `important ` also in dynamic styles
                    set_css_property(&declaration, &name, &value, false);
                } else {
                    declaration
                        .remove_property(&name)
                        .expect_throw("style: remove_property failed");
                }
            });
            if droppable {
                task_handles.push(Task::start_droppable(task));
            } else {
                Task::start(task);
            }
        }
        (rule_id_and_index, task_handles)
    }

    fn remove_rule(&self, id: u32) {
        let (rule_index, _ids_lock) = self.rule_ids.remove_id(id);
        self.sheet
            .delete_rule(u32::try_from(rule_index).expect_throw("style: rule_index casting failed"))
            .expect_throw("style: delete_rule failed");
    }
}

fn set_css_property(declaration: &CssStyleDeclaration, name: &str, value: &str, important: bool) {
    // @TODO refactor?

    let priority = if important { "important" } else { "" };

    match declaration.set_property_with_priority(name, value, priority) {
        Ok(declaration) => declaration,
        Err(error) => {
            // e.g. `CSSStyleDeclaration.setProperty: Can't set properties on
            // CSSFontFaceRule declarations` on Firefox
            crate::eprintln!("{:#?}", error);
            return;
        }
    }

    if not(declaration
        .get_property_value(name)
        .expect_throw("style: get_property_value failed")
        .is_empty())
    {
        return;
    }
    for name_prefix in iter::once("").chain(VENDOR_PREFIXES) {
        let prefixed_name = [name_prefix, name].concat();
        for value_prefix in iter::once("").chain(VENDOR_PREFIXES) {
            let prefixed_value = [value_prefix, value].concat();
            declaration
                .set_property_with_priority(&prefixed_name, &prefixed_value, priority)
                .expect_throw("style: set_property_with_priority failed");
            if not(declaration
                .get_property_value(&prefixed_name)
                .expect_throw("style: get_property_value failed")
                .is_empty())
            {
                return;
            }
        }
    }
    panic!("invalid CSS property: `{}: {};`", name, value);
}
