It will go in the order of:
Workspace Rule > Workspace Default > Monitor Rule > Monitor Default > Default

If you set a count to 2 it means that that offset will be used for up to and including 2 windows. The lowest count below or at current windows will be used. If you open more windows it will instead use the Workspace Default or the next level of specificity one in line that is deifned.

TODO: Write some instructions on how to actually use it. For now refer to either the example.json or the code.

BUGS: Does not automatically apply the new offset whenever you move a window to another workspace, you have to switch focus to another window first.
