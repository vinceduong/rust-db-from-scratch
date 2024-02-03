# rust-db-from-scratch
Let's create a non relationnal schemed document oriented database system, with a rich type system and code first for a safe developer experience.

In the end, the database schemes and migration definitions should be rust code, and migrations should happen at startup time.

A query to the database should be types, there should be no data validation needed when fetching documents.

## Storage

Collection data is stored in files, distributed in blocks of a size of X bytes.

The blocks can contain a different amount of documents, since there is no fixed size for them.

Indexes will be stored in indexes files, telling the block location given an indexed field value.
