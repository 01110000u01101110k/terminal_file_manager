# Console file manager

I wrote my own simple console file manager. It was conceived as a simplified version of 'Far Manager'. The purpose of this project was to learn and practice such modules of the standard library as std::fs and std::io in more depth, and I also learned how to work with the crossterm library.

The following features are currently implemented in the project:
- Moving through the file system using the arrow keys, enter and esc. And also move between items by pressing the left mouse button on an item. Double-click to navigate between folders or open files.
- Context menu, which is called by pressing the right mouse button. Menu items are activated by pressing the Enter key. The following context menu items are implemented: delete, create directory, create file, copy, paste, rename, information, copy path.

The code in the project is still a bit sloppy, there are a lot of unwraps and some errors are not handled, and there are at least a few minor bugs.

The project was tested on Windows. But it should also be supported on Linux systems (I haven't tested it yet).