# VHS Tape: Issue List Demo
# Run with: vhs issue-list.tape

# Configuration
Output issue-list.gif
Set FontSize 14
Set Width 1200
Set Height 800
Set Theme "Catppuccin Mocha"
Set Padding 20
Set Framerate 30

# Terminal setup
Set TypingSpeed 50ms

# Start the application
Type "lazyjira"
Enter
Sleep 2s

# Navigate down through issues
Sleep 500ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Navigate up
Type "k"
Sleep 300ms
Type "k"
Sleep 500ms

# Jump to top with gg
Type "gg"
Sleep 500ms

# Jump to bottom with G
Type "G"
Sleep 500ms

# Back to top
Type "gg"
Sleep 500ms

# Show the help panel briefly
Type "?"
Sleep 2s
Type "q"
Sleep 500ms

# Open JQL input
Type ":"
Sleep 500ms
Type "assignee = currentUser()"
Sleep 1s
Escape
Sleep 500ms

# Quit
Type "q"
Sleep 500ms
