# Project Overwatch Scanner

The scanner component of Project Overwatch, for which the you can find the frontend on [Github](https://github.com/Stetsed/project-overwatch-frontend). This component takes a JSON configuration file for the arguments and a .env file for the secure variables. Then it requests the data from the dynmap endpoint specified which is taken from the JSON file, and then scrapes time = 0 from there which will give the most up to date information. Then it takes this information and sends it to both the Pocketbase database, which is used by the frontend and the mysql database which is currently legacy but is needed as the Rust Pocketbase SDK I am using which can be found [here](https://github.com/sreedevk/pocketbase-sdk-rust), doesn't support the required requests such as mass deletion with a filter although this might be a limitation of pocketbase itself.

The program checks if the players are in any of the regions specified in the JSON file and if they are it sends a message to the discord and saves them to the active folder of the database, then it checks if they are in the active database and are still in the region, in this case it just updates the active database and lastly if they are no longer in the region it will send a message to the discord that they have left.

## Technologies

Backend: [MySQL](https://github.com/mysql/mysql-server) & [PocketBase](https://github.com/pocketbase/pocketbase)

SDK: [Discord](https://github.com/serenity-rs/serenity) & [PocketBase](https://github.com/sreedevk/pocketbase-sdk-rust) & [MySQL](https://github.com/launchbadge/sqlx)
## Roadmap

- [x] Add support for overworld dimension.
- [x] Integrate with Discord to send Alerts through a bot
- [x] Add pocketbase support
- [ ] Add support for the nether dimension.
- [ ] Move over completley from MySQL to PocketBase.
