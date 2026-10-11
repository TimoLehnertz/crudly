# Row lineage review: open bugs

Review of `graphite-backend` branch `feat/row-lineage` (commits `90908ed9`, `e7dc5cb3`, `a3e72659`).

## 1. Map node breaks the lineage chain

**Where:** `graphite-engine/src/graph/node/executable_node/table_nodes/gt_map_node.rs`,
`graphite-engine/src/graph/node/executable_node/table_nodes/game_table_items/build_row.rs`

The Map node passes each row through a closure that usually splits it with SeparateRow and rebuilds it with
BuildRow. `BuildRowNode` builds the new row with `Row::new`, which starts with an empty `Origin`. When the graph
executor attributes it, the row gets a `Create` step from the inner BuildRow node, and its whole history before the
Map node is lost.

The doc comment on `Row` says rows can only be created by `graphite_table` so their `Origin` stays truthful, but
`Row::new` is public and nodes use it to rebuild rows.

**Effect:** every table after a Map node shows its rows as created from scratch, with no parent table.

**Suggested fix:** in `GTMapNode`, carry the input row's `Origin` over to the mapped row. The closure can't change the
competitor, so the input and output rows match up one to one. Add a crate API for this, for example
`Row::with_origin_of(&mut self, source: &Row)`, or record a `Map` step. Add a lineage test that runs a table through
a Map node.

## 2. Lineage is written with one INSERT per entry

**Where:** `graphite-engine/src/dao/row_lineage_dao.rs`, `RowLineageDAO::replace_for_table`

`replace_for_table` runs a separate `INSERT` for every row and every trail. It runs every time a node writes a game
table or a standalone table (`set_game_table` and `set_standalone_table` in `node_api.rs`). `RowDAO::insert_many`
writes the same rows in batches.

**Effect:** on large events, every graph execution that rewrites tables pays one SQLite statement per lineage entry,
which roughly doubles the write cost of persisting tables.

**Suggested fix:** batch the inserts the same way as `RowDAO::insert_many`: multi-row `VALUES` lists chunked to stay
under SQLite's bound-parameter limit (4 parameters per entry), or a single prepared statement reused in a loop.
