Limitations
-----------

In general Glulxtoc is likely to have problems with any Glulx files that weren't compiled with Inform.

- No functions in RAM
- No 1 and 2 byte locals
- Computed branch and jump offsets are only supported when you supply an Inform debug file
- Inter-function branches are only supported when you manually set the target function as unsafe