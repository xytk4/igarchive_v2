# igarchive_v2
Old Instagram Message Browser/Console nonsense

Get a messages.json from the Instagram data dump zip you can get in your account page somewhere...
Then this will turn it into a browsable console you can... browse through.
Also do stuff like text search, statistics, that sort of thing.
Written in rust because that's the thing to do these days, and it makes it easier to make it actually work.
```
Help text!
Normal mode commands:
 *   t: change thread // 't n' where n is thread id to change to.
 *  ti: thread info // 'ti' for current thread or 'ti n' where n is thread id to display info for
 *  ct: current thread // display currently selected thread id
 *   p: participants // list all participants in currently selected thread
 * lat: list all threads // lists all threads in the archive
 *   n: find name changes // lists all name changes that happened in the currently selected thread
 * ams: advanced message stats // produces a leaderboard of # messages sent sorted by message count
 * ams-mot: messages over time // 'ams-mot i' where i is interval (d, m, y); outputs message count at that interval
 * ams-dt: message delta-time // calculates the shortest and longest time between messages in the thread
 * gf: global find, same as find but for all threads at once // 'gf blah blah'
 * gfu: global find by user: same as the above but for user // 'gfu username'
 * m!: switch to the message-mode console, which lets you look at messages and perform per-message ops.
 * fts: perform first time setup again.
 * q: quit

Message-mode commands:
 * (. = +): next message
 * (, - _): prev message
 * s: go to first message
 * e: go to latest message
 * af: toggle auto-forward (advances message index on empty console input)
 * raf: toggle reverse-auto-forward (reverses auto-forward direction, for compatibility reasons or something)
 * m: view media info // 'm' for current message or 'm n' where n is message id
 * l: view message likes // 'l' for current message or 'l n' where n is message id
 * f: find text in thread messages: 'f blah blah ...'
 * fu: find by user: outputs all of user's messages in thread // 'fu username'
 * q: quit message mode, return to normal console
 ```
 
 Bonus tips:
 - It outputs nice colo(u)red text and tables and things, so you might want to use the new fancy Windows Terminal or something that supports color and maybe unicode
 - It probably works on Linux, I haven't bothered to check. It definitely works on Windows, though, probably.
 
 First time setup:
    The first time you run the program, it will ask for the location of messages.json, as well as your username. It uses the username to exclude you from 
    thread participant lists, so it needs it to be correct. The FTS information is stored in a file called ia_settings.json, in the same folder as the program.
 
 
