# Enygma (Nuevahacks project)
## Brief description
As the current school system has been upended, it is very important to explore novel ways we can exercise our minds independently of whatever systems existed previously. Created in response to this need, Enygma is a social-media-esque platform for sharing and creating interesting problems. 

## Infrastructure
* The client
  * Located in /client of this repo
  * This is the frontend of the application
  * Written in Rust using yew
* The main server
  * Located in /server of this repo
  * Hosts the client application as well as the public api surface for Enygma
  * Written in rust using actix-web
* The PostgreSQL server
  * This is hosted privately and is only interfaced by the main server 
  * Hosts all schema and models required byt he application such as users and problems
* The recommendations server
  * This interacts with the main server to generate recommendations for each user based on a variety of factors
  * Located at [https://github.com/countableclouds/rec_system](https://github.com/countableclouds/rec_system)

## RoadMap (Things we need to get done)
- [ ] Infrastructure
  * Implement infrastructure plans to connect recommendation server to main server
- [ ] Improve the UI
  * Currently the CSS could use some improving
- [ ] Profile page
  * View and follow users' profiles
- [ ] Vetting system
  * Allow problem owners to vett solutions to their problems
- [ ] Discussion system
  * Allow users to discuss solutions
- [ ] Difficulty system
  * Have problem difficulty dynamically decided rather than statically by the problem owner
- [ ] Creation studio
  * Improve the problem creation studio to allow creation collections of problems and editing problems
- [ ] Security
  * Currently there aren't any sophisticated security measures on the server (no protections against botting, etc)

etc