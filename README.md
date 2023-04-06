# Rust, Rocket, SQLite, and Forms

This project is an exploration of Rust, [Rocket](https://rocket.rs/), SQLite, and forms to learn more about their integration and usage. This learning exercise covers various aspects, such as displaying user information, listing projects, and more.

The following routes have been implemented:

[All Users](http://127.0.0.1:8000/all-users) - Displays a list of all users in the database.

[All Projects](http://127.0.0.1:8000/all-projects) - Shows a list of all projects in the database.

[All Projects for User `<id>`](http://127.0.0.1:8000/all-projects-for-user/4) - Lists all projects associated with a specific user, identified by their user ID.

[User Info and All Projects for That User](http://127.0.0.1:8000/user/1) - Displays detailed information for a specific user, along with a list of all their associated projects.

[Add User](http://127.0.0.1:8000/add-user) - A form to add a new User to the database.

[Add Project](http://127.0.0.1:8000/add-user) - A form to add a new Project for a User to the database.

While the implementation might not be perfect, the primary goal of this project is to learn and experiment with the various technologies involved.
