# hearthglow

Hearthglow is a program that runs a linux native UI application.
The purpose of this program is to track family calendar events, wins, reminders and more.
It is designed to be run on a local network with ssh access for remote control.


## Design

## Code 
```
Configuration -> Used to bootstrap the application. ("configuration.hg")
Types -> Defines the data types used in the application.
CLI -> Command Line Interface for local control (includes checking config, making edits to the database).
App -> The primary application logic loop is event driven; it handles user input and updates the UI accordingly, and storing updates in the database. It also is triggered on timed events in the database.
```
## Architecture
```
--------
| UX   |
--------
   |
   |
--------       -----------
| App   | ---> | Database |  
--------       -----------
   |
   |
--------
| CLI   |
--------

```
