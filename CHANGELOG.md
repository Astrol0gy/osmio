# Unreleased

* Refactor PBF objects internally to be more effecient

# v0.2.0 (2020-01-06)

* ISO timeformats are more clearer without milliseconds
* `OSMObjType` now implemented `Display` (`"node"`/`"way"`/`"relation"`) and `Debug` implemention changed to be single character representing type (`"n"`/`"w"`/`"r"`), and are sortable.
* `tags` (& `members`) for objects are now an `ExactSizedIterator`
* New helper methods: `tagged` & `untagged`
* Object readers now have `.inner()`, returning a ref to their inner `Read` object they're reading from (not fully implemented)

# v0.1.0 (2019-10-06)

* Initial work
