# Merging a Table

Delta Lake `MERGE` operations allow you to merge source data into a target table based on specific conditions. `MERGE` operations are great for making selective changes to your Delta table without having to rewrite the entire table.

Use [`dt.merge()`][deltalake.DeltaTable.merge] with a single or multiple conditional statements to perform CRUD operations (Create, Read, Update, Delete) at scale. You can also use Delta Lake merge for efficient Change Data Capture (CDC), Slowly Changing Dimensions (SDC) operations and to ensure GDPR compliance.

## Basic Structure of `MERGE` Command

Let’s start by understanding the basic structure of a Delta Lake `MERGE` command in delta-rs.

Use the [TableMerger API](https://delta-io.github.io/delta-rs/api/delta_table/delta_table_merger/) to construct a `MERGE` command with one or multiple conditional clauses:

```python
    (
        dt.merge(                                       # target data
            source=source_data,                         # source data
            predicate="target.x = source.x",
            source_alias="source",
            target_alias="target")
        .when_matched_update(                           # conditional statement
            updates={"x": "source.x", "y":"source.y"})
        .execute()
    )
```

In the syntax above:

- `dt` is your target Delta table
- `source_data` is your source data
- `source` and `target` are your respective aliases for your source and target datasets
- `when_matched_update` is one of many possible conditional statements, see the sections below for more
- `execute()` executes the MERGE operation with the specified settings

Note that executing a `MERGE` operation automatically writes the changes to your Delta table in a single transaction.

## Update

Use [`when_matched_update`][deltalake.table.TableMerger.when_matched_update] to perform an UPDATE operation.

You can define the rules for the update using the `updates` keyword. If a `predicate` is passed to `merge()` then only rows which evaluate to true will be updated.

For example, let’s update the value of a column in the target table based on a matching row in the source table.

=== "Python"
    ```python
    from deltalake import DeltaTable, write_deltalake
    import pyarrow as pa

    # define target table
    > target_data = pa.table({"x": [1, 2, 3], "y": [4, 5, 6]})
    > write_deltalake("tmp_table", target_data)
    > dt = DeltaTable("tmp_table")
    > dt.to_pandas().sort_values("x", ignore_index=True)

    x  y
    0  1  4
    1  2  5
    2  3  6

    # define source table
    > source_data = pa.table({"x": [2, 3], "y": [5,8]})
    > source_data

    x  y
    0  2  5
    1  3  8

    # define merge logic
    > (
    >     dt.merge(
    >         source=source_data,
    >         predicate="target.x = source.x",
    >         source_alias="source",
    >         target_alias="target")
    >     .when_matched_update(
    >         updates={"x": "source.x", "y":"source.y"})
    >     .execute()
    > )
    ```

=== "Rust"
    ```rust
    // define target table
    let delta_ops = DeltaOps::try_from_uri("tmp/some-table").await?;
    let mut table = delta_ops
        .create()
        .with_table_name("some-table")
        .with_save_mode(SaveMode::Overwrite)
        .with_columns(
            StructType::new(vec![
                StructField::new(
                    "x".to_string(),
                    DataType::Primitive(PrimitiveType::Integer),
                    true,
                ),
                StructField::new(
                    "y".to_string(),
                    DataType::Primitive(PrimitiveType::Integer),
                    true,
                ),
            ])
            .fields()
            .cloned(),
        )
        .await?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("x", arrow::datatypes::DataType::Int32, true),
        Field::new("y", arrow::datatypes::DataType::Int32, true),
    ]));
    let mut record_batch_writer = deltalake::writer::RecordBatchWriter::for_table(&mut table)?;
    record_batch_writer
        .write(RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                Arc::new(Int32Array::from(vec![4, 5, 6])),
            ],
        )?)
        .await?;

    record_batch_writer.flush_and_commit(&mut table).await?;

    let ctx = SessionContext::new();
    let source_data = ctx.read_batch(RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int32Array::from(vec![2, 3])),
            Arc::new(Int32Array::from(vec![5, 6])),
        ],
    )?)?;

    DeltaOps(table)
        .merge(source_data, "target.x = source.x")
        .with_source_alias("source")
        .with_target_alias("target")
        .when_matched_update(|update| 
            update
            .update("x", "source.x")
            .update("y", "source.y"))?
        .await?;
    ```

First, we match rows for which the `x` values are the same using `predicate="target.x = source.x"`. We then update the `x` and `y` values of the matched row with the new (source) values using `updates={"x": "source.x", "y":"source.y"}`.

```python
# inspect result
> dt.to_pandas().sort_values("x", ignore_index=True)

   x  y
0  1  4
1  2  5
2  3  8
```

The value of the `y` column has been correctly updated for the row that matches our predicate.

You can also use [`when_matched_update_all`][deltalake.table.TableMerger.when_matched_update_all] to update all source fields to target fields. In this case, source and target are required to have the same field names.

## Insert

Use [`when_not_matched_insert`][deltalake.table.TableMerger.when_not_matched_insert] to perform an INSERT operation.

For example, let’s say we start with the same target table:

=== "Python"
    ```python
    > target_data = pa.table({"x": [1, 2, 3], "y": [4, 5, 6]})
    > write_deltalake("tmp_table", target_data)
    > dt = DeltaTable("tmp_table")

    x  y
    0  1  4
    1  2  5
    2  3  6
    ```
=== "Rust"
    ```rust
    let delta_ops = DeltaOps::try_from_uri("./data/simple_table").await?;
    let mut table = delta_ops
        .create()
        .with_table_name("some-table")
        .with_save_mode(SaveMode::Overwrite)
        .with_columns(
            StructType::new(vec![
                StructField::new(
                    "x".to_string(),
                    DataType::Primitive(PrimitiveType::Integer),
                    true,
                ),
                StructField::new(
                    "y".to_string(),
                    DataType::Primitive(PrimitiveType::Integer),
                    true,
                ),
            ])
            .fields()
            .cloned(),
        )
        .await?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("x", arrow::datatypes::DataType::Int32, true),
        Field::new("y", arrow::datatypes::DataType::Int32, true),
    ]));
    let mut record_batch_writer = deltalake::writer::RecordBatchWriter::for_table(&mut table)?;
    record_batch_writer
        .write(RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                Arc::new(Int32Array::from(vec![4, 5, 6])),
            ],
        )?)
        .await?;

    record_batch_writer.flush_and_commit(&mut table).await?;
    ```

And we want to merge only new records from our source data, without duplication:

=== "Python"
    ```python
    > source_data = pa.table({"x": [2,3,7], "y": [4,5,8]})

    x  y
    0  2  5
    1  3  6
    2  7  8
    ```
=== "Rust"
    ```rust
        let ctx = SessionContext::new();
    let source_data = ctx.read_batch(RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int32Array::from(vec![2, 3])),
            Arc::new(Int32Array::from(vec![5, 6])),
        ],
    )?)?;
    ```


The `MERGE` syntax would be as follows:

=== "Python"
    ```python
    (
        dt.merge(
            source=source_data,
            predicate="target.x = source.x",
            source_alias="source",
            target_alias="target")
        .when_not_matched_insert(
            updates={"x": "source.x", "y":"source.y"})
        .execute()
    )

    > # inspect result
    > print(dt.to_pandas().sort_values("x", ignore_index=True))

    x  y
    0  1  4
    1  2  5
    2  3  6
    3  7  8
    ```

=== "Rust"
    ```rust
    DeltaOps(table)
    .merge(source_data, "target.x = source.x")
    .with_source_alias("source")
    .with_target_alias("target")
    .when_not_matched_insert(
        |insert| insert.set("x", "source.x").set("y", "source.y")
    )?.await?;
    ```

The new row has been successfully added to the target dataset.

You can also use [`when_not_matched_insert_all`][deltalake.table.TableMerger.when_not_matched_insert_all] to insert a new row to the target table, updating all source fields to target fields. In this case, source and target are required to have the same field names.

## Delete

Use [`when_matched_delete`][deltalake.table.TableMerger.when_matched_delete] to perform a DELETE operation.

For example, given the following `target_data` and `source_data`:

```python
target_data = pa.table({"x": [1, 2, 3], "y": [4, 5, 6]})
write_deltalake("tmp_table", target_data)
dt = DeltaTable("tmp_table")
source_data = pa.table({"x": [2, 3], "deleted": [False, True]})
```

You can delete the rows that match a predicate (in this case `"deleted" = True`) using:

=== "Python"
    ```python
    (
        dt.merge(
            source=source_data,
            predicate="target.x = source.x",
            source_alias="source",
            target_alias="target")
        .when_matched_delete(
            predicate="source.deleted = true")
        .execute()
    )
    ```
=== "Rust"
    ```rust
    DeltaOps(table)
    .merge(source_data, "target.x = source.x")
    .with_source_alias("source")
    .with_target_alias("target")
    .when_matched_delete(
        |delete| delete.predicate("source.deleted = true")
    )?.await?;
    ```

This will result in:

```python
> dt.to_pandas().sort_values("x", ignore_index=True)

   x  y
0  1  4
1  2  5
```

The matched row has been successfully deleted.

## Upsert

You can combine conditional statements to perform more complex operations.

To perform an upsert operation, use `when_matched_update` and `when_not_matched_insert` in a single `merge()` clause.

For example:

=== "Python"

    ```python
    target_data = pa.table({"x": [1, 2, 3], "y": [4, 5, 6]})
    write_deltalake("tmp_table", target_data)
    dt = DeltaTable("tmp_table")
    source_data = pa.table({"x": [2, 3, 5], "y": [5, 8, 11]})

    (
        dt.merge(
            source=source_data,
            predicate="target.x = source.x",
            source_alias="source",
            target_alias="target")
        .when_matched_update(
            updates={"x": "source.x", "y":"source.y"})
        .when_not_matched_insert(
            updates={"x": "source.x", "y":"source.y"})
        .execute()
    )
    ```
=== "Rust"
    ```rust
    DeltaOps(table)
    .merge(source_data, "target.x = source.x")
    .with_source_alias("source")
    .with_target_alias("target")
    .when_matched_update(
        |update| update.update("x", "source.x").update("y", "source.y"),
    )?
    .when_not_matched_insert(
        |insert| insert.set("x", "source.x").set("y", "source.y"),
    )?
    .await?;
    ```

This will give you the following output:

```python
> dt.to_pandas().sort_values("x", ignore_index=True)

   x  y
0  1  4
1  2  5
2  3  8
3  5  11
```

## Upsert with Delete

Use the [`when_matched_delete`][deltalake.table.TableMerger.when_matched_delete] or [`when_not_matched_by_source_delete`][deltalake.table.TableMerger.when_not_matched_by_source_delete] methods to add a DELETE operation to your upsert. This is helpful if you want to delete stale records from the target data.

For example, given the same `target_data` and `source_data` used in the section above:

=== "Python"
    ```python
    (
        dt.merge(
            source=source_data,
            predicate="target.x = source.x",
            source_alias="source",
            target_alias="target")
        .when_matched_update(
            updates={"x": "source.x", "y":"source.y"})
        .when_not_matched_insert(
            updates={"x": "source.x", "y":"source.y"})
        .when_not_matched_by_source_delete()
        .execute()
    )
    ```
=== "Rust"
    ```rust
    DeltaOps(table)
    .merge(source_data, "target.x = source.x")
    .with_source_alias("source")
    .with_target_alias("target")
    .when_matched_update(
        |update| update.update("x", "source.x").update("y", "source.y"),
    )?
    .when_not_matched_insert(
        |insert| insert.set("x", "source.x").set("y", "source.y"),
    )?
    .when_not_matched_by_source_delete(|delete| delete)?
    .await?;
    ```


This will result in:

```python
> dt.to_pandas().sort_values("x", ignore_index=True)

   x  y
0  2  5
1  3  8
2  5  11
```

The row containing old data no longer present in the source dataset has been successfully deleted.

## Multiple Matches

Note that when multiple match conditions are met, the first condition that matches is executed.

For example, given the following `target_data` and `source_data`:

```python
target_data = pa.table({"x": [1, 2, 3], "y": [4, 5, 6]})
write_deltalake("tmp_table", target_data)
dt = DeltaTable("tmp_table")
source_data = pa.table({"x": [2, 3, 5], "y": [5, 8, 11]})
```

Let’s perform a merge with `when_matched_delete` first, followed by `when_matched_update`:

```python
(
    dt.merge(
        source=source_data,
        predicate="target.x = source.x",
        source_alias="source",
        target_alias="target")
    .when_matched_delete(
        predicate="source.x = target.x")
    .when_matched_update(
        updates={"x": "source.x", "y":"source.y"})
    .execute()
)
```

This will result in:

```python
> dt.to_pandas().sort_values("x", ignore_index=True)

   x  y
0  1  4
```

Let’s now perform the merge with the flipped order: `update` first, then `delete`:

```python
(
    dt.merge(
        source=source_data,
        predicate="target.x = source.x",
        source_alias="source",
        target_alias="target")
    .when_matched_update(
        updates={"x": "source.x", "y":"source.y"})
    .when_matched_delete(
        predicate="source.x = target.x")
    .execute()
)
```

This will result in:

```python
> dt.to_pandas().sort_values("x", ignore_index=True)

   x  y
0  1  4
1  2  5
2  3  8
```

## Conditional Updates

You can perform conditional updates for rows that have no match in the source data using [when_not_matched_by_source_update][deltalake.table.TableMerger.when_not_matched_by_source_update].

For example, given the following target_data and source_data:

```python
target_data = pa.table({"1 x": [1, 2, 3], "1y": [4, 5, 6]})
write_deltalake("tmp", target_data)
dt = DeltaTable("tmp")
source_data = pa.table({"x": [2, 3, 4]})
```

Set y = 0 for all rows that have no matches in the new source data, provided that the original y value is greater than 3:

```python
(
   dt.merge(
       source=new_data,
       predicate='target.x = source.x',
       source_alias='source',
       target_alias='target')
   .when_not_matched_by_source_update(
       predicate = "`1y` > 3",
       updates = {"`1y`": "0"})
   .execute()
)
```

This will result in:

```python
> dt.to_pandas().sort_values("x", ignore_index=True)
   x  y
0  1  0
1  2  5
2  3  6
```

## Notes

- Column names with special characters, such as numbers or spaces should be encapsulated in backticks: "target.`123column`" or "target.`my column`"

## Optimizing Merge Performance

Delta Lake merge operations can be resource intensive when working with large tables. Processing time and compute resources can be significantly reduced by using
efficient predicates which are specific to the data being merged.

The following strategies explain how to optimize merge operations for better performance.

#### 1. Add Partition Columns to Predicates

When your table is partitioned, including partition columns in your merge predicate can improve performance by enabling file pruning:

```python
# Good - uses partition column and value
(
    dt.merge(
        source=source_data,
        predicate="s.id = t.id AND s.month_id = t.month_id AND t.month_id = 202501",  # month_id is a partition column
        source_alias="s",
        target_alias="t")
    .execute()
)


# Less optimal - no partition column usage
(
    dt.merge(
        source=source_data,
        predicate="s.id = t.id",  # No partition column specified
        source_alias="s",
        target_alias="t")
    .execute()
)

```

As you can see, your filter should specify the partition column(s) and the value(s) you want to target during the merge operation.
This is especially important when using the default argument `streamed_exec=True` in the `merge` method which disables the use of source table statistics to derive an early pruning predicate.
Without these statistics, explicit predicates in your merge condition are required for file pruning.

#### 2. Add Additional Filter Columns to Predicates

Partitioning data on certain columns may be inefficient when it creates an excessive number of files or results in files that are too small.

For example, you might have a source which is updated daily with the previous date's data. If the size of the data is too small to justify daily
partitioning, you can use the following predicate to prune the monthly partition and only join the previous day's data:

```python
# The predicate below prunes the monthly partition and only joins the previous day's data
# Note: the table is only partitioned by the `month_id` column. The `date_id` value is computed from the source data.
(
    dt.merge(
        source=source_data,
        predicate="s.id = t.id AND s.month_id = t.month_id AND t.month_id = 202501 AND s.date_id = t.date_id AND t.date_id = 20250120",
        source_alias="s",
        target_alias="t")
    .execute()
)

```

### Performance Impact

The effectiveness of these optimizations can be monitored using the operation metrics:

```python
metrics = dt.merge(...).execute()

print(f"Files scanned: {metrics.get('num_target_files_scanned')}")
print(f"Files skipped: {metrics.get('num_target_files_skipped_during_scan')}")
print(f"Execution time: {metrics.get('execution_time_ms')} ms")
```

An efficient merge operation should show:

- A high ratio of skipped files to scanned files
- Lower execution time compared to less specific predicates

Here is an example of logs from a merge operation with a partitioned table without adding the partition columns to the predicates:

```text
Merging table with predicates: {
    'merge': 's.unique_constraint_hash = t.unique_constraint_hash',
    'when_matched_update_all': 's.post_transform_row_hash != t.post_transform_row_hash'
}
Files Scanned: 24
Files Skipped: 0
Files Added: 1
Execution Time: 23774ms
```

The table is partitioned by the `month_id` column and all files are scanned which is not efficient. If we add the partition columns to the predicates,
the merge operation will only scan the relevant files which is faster and more efficient:

```text
Merging table with predicates: {
    'merge': 's.unique_constraint_hash = t.unique_constraint_hash AND s.month_id = t.month_id AND t.month_id = 202503',
    'when_matched_update_all': 's.post_transform_row_hash != t.post_transform_row_hash AND s.month_id = t.month_id AND t.month_id = 202503'
}
Files Scanned: 1
Files Skipped: 10
Files Added: 1
Execution Time: 2964ms
```

For this specific source, it is known that each data update only includes a few recent days. This means that we can further optimize the merge operation
by making the predicate even more specific by adding the unique `date_id` column values to the predicates.

```text
 Merging table with predicates: {
    'merge': 's.unique_constraint_hash = t.unique_constraint_hash AND s.month_id = t.month_id AND t.month_id = 202503 AND s.date_id = t.date_id AND t.date_id IN (20250314, 20250315, 20250316)',
    'when_matched_update_all': 's.post_transform_row_hash != t.post_transform_row_hash AND s.month_id = t.month_id AND t.month_id = 202503 AND s.date_id = t.date_id AND t.date_id IN (20250314, 20250315, 20250316)'
}
Files Scanned: 0
Files Skipped: 18
Files Added: 1
Execution Time: 416ms
```

The final result skips all files that are not relevant to the merge operation and is significantly faster (98%) than the original operation.

### Best Practices

1. **Always Include Partition Columns**: If your table is partitioned, include partition columns in your merge predicates
2. **Keep Statistics Updated**: Regular table optimization helps Delta Lake make better decisions about file pruning
3. **Monitor Metrics**: Use the operation metrics to verify the effectiveness of your predicates
