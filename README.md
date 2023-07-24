# Rust-Sqlx

## To use it you have to 

Install docker (Ubuntu/Debian) 
```
sudo snap install docker
```

Run docker with user and password
```
sudo docker run -e POSTGRES_PASSWORD=mysecretpassword -e POSTGRES_USER=dbuser -e POSTGRES_DB=bookstore -p 5432:5432 postgres
```
If you'd like to change passsword and user you have to change the pools variables on src/main.rs

### Thanks !
