This allows you to set custom work area offsets for workspaces and/or monitors depending on the number of active windows in your active workspaces. 

It will go in the order of:
Workspace Rule > Workspace Default > Monitor Rule > Monitor Default > Default

If you set a count to 2 it means that that offset will be used for up to and including 2 windows. The lowest count below or at current windows will be used. If you open more windows it will instead use the Workspace Default or the next level of specificity one in line that is deifned.

Demo:
https://github.com/user-attachments/assets/44db7e7b-f3a7-4e93-b7d3-6f97a266c6cb

TODO: Write some instructions on how to actually use it. For now refer to either the example.json or the code.




