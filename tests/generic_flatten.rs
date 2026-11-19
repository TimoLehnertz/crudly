//! Generic structs with a flattened type parameter: the derived impls carry the bounds they need,
//! so the struct itself declares none.

use crudly::{HasColumns, HasId, IntoRow, Schema};

#[derive(IntoRow, Schema)]
#[crudly(table = "shelves")]
struct Shelf<Item> {
    #[crudly(id)]
    id: i64,
    #[crudly(flatten)]
    item: Item,
}

#[derive(IntoRow)]
struct Kite {
    wingspan: i64,
}

/// Has no row mapping and is not even `Send`.
struct Pebble(std::rc::Rc<()>);

#[test]
fn the_traits_are_implemented_when_the_type_argument_has_columns() {
    assert_eq!(<Shelf<Kite> as Schema>::table_name(), "shelves");
    assert_eq!(<Shelf<Kite> as Schema>::columns(), vec!["id", "wingspan"]);
    assert_eq!(<Shelf<Kite> as HasColumns>::columns(), vec!["wingspan"]);
    assert_eq!(<Shelf<Kite> as HasId>::id_column(), "id");
}

/// With a type argument that has no columns the struct is still a struct. That it then implements
/// neither `Schema` nor `HasColumns` is the `compile_fail` example in the crate docs.
#[test]
fn a_type_argument_without_columns_leaves_a_plain_struct() {
    let shelf = Shelf {
        id: 4,
        item: Pebble(Default::default()),
    };
    assert_eq!(HasId::id(&shelf), 4);
    let Pebble(_) = shelf.item;
}

#[test]
fn bounds_written_on_the_struct_still_compile() {
    #[derive(IntoRow, Schema)]
    struct Drawer<Item: HasColumns + Send> {
        label: String,
        #[crudly(flatten)]
        item: Item,
    }

    assert_eq!(
        <Drawer<Kite> as Schema>::columns(),
        vec!["label", "wingspan"]
    );
}

#[test]
fn the_id_may_be_a_type_parameter() {
    #[derive(IntoRow, Schema)]
    #[crudly(table = "hooks", external_ids)]
    struct Hook<Id> {
        #[crudly(id)]
        id: Id,
        load: i64,
    }

    assert_eq!(<Hook<String> as Schema>::columns(), vec!["id", "load"]);
}
