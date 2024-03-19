# mmp
my music player


## layout
### mmp-server 
server for mmp. 

--- 
inner workings: 
- use sqlite for long term data - users, playlists and song metadata
- use a in-memory map for songs, and other transient data
- axum for web server. still not sure on layout of routes
### mmp-lib
common utilities for mmp
### mmp-client
test cli client

