use warp::Filter;
use tokio_postgres::{NoTls, Error, Client};
use std::sync::Arc;
use warp::reject::custom;

// Define um tipo de erro personalizado que implementa Reject
#[derive(Debug)]
struct CustomError(String);

impl warp::reject::Reject for CustomError {}

// Define uma estrutura de dados para o item
#[derive(serde::Deserialize, serde::Serialize)]

struct Task {
    id: i32,
    title: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), Error>{
    let (client, connection) =
    tokio_postgres::connect("host=localhost user=postgres password=1234 dbname=postgres", NoTls)
        .await?;
tokio::spawn(connection);

let client = Arc::new(client);

let db = warp::any().map(move || client.clone());


let cors = warp::cors()
.allow_any_origin() // Permitir qualquer origem (modificar conforme necessário)
.allow_methods(vec!["GET", "POST", "DELETE"]) // Métodos permitidos
.allow_headers(vec!["Content-Type"]) // Cabeçalhos permitidos
.max_age(3600); // Tempo máximo de cache para as opções pré-voo

let create_task = warp::post()
.and(warp::path("tasks"))
.and(warp::body::json())
.and(db.clone())
.and_then(|task: Task, client: Arc<Client>| async move {
    let insert_query = format!("INSERT INTO tasks (title, description) VALUES ('{}', '{}')", task.title, task.description);
    match client.execute(&insert_query, &[]).await {
        Ok(rows) if rows == 1 => {
            Ok(warp::reply::json(&task))
        }
    _ => {
        let error_message = "failed to add task".to_string();
        Err(custom(CustomError(error_message)))
    },
}
});

let get_tasks = warp::get()
.and(warp::path("tasks"))
.and(db.clone())
.and_then(|client: Arc<Client> | async move {
    let query = format!("SELECT id, title, description FROM tasks");

        match client.query(&query, &[]).await {
            Ok(rows) => {
                let tasks: Vec<Task> = rows
                .into_iter()
                .map(|row | Task {
                    id: row.get("id"),
                    title: row.get("title"),
                    description: row.get("description"),
                })
                .collect();

            Ok(warp::reply::json(&tasks))
            }
            Err(err) => {
                let error_message = format!("Error to fetch tasks: {}", err);
                Err(custom(CustomError(error_message)))
            }
        }
});

let delete_task = warp::delete()
.and(warp::path!("tasks" / i32))
.and(db.clone())
.and_then(|task_id: i32, client: Arc<Client>| async move {
    let delete_query = format!("DELETE FROM tasks WHERE id = {}", task_id);

    match client.execute(&delete_query, &[]).await {
        Ok(rows) if rows == 1 => {
            Ok(warp::reply::html("success in delete task"))
        } 
        _ => {
            let error_message = format!("Error in delete task: with id: {}", task_id);
            Err(custom(CustomError(error_message)))
        }
    }
});


let routes = create_task.or(get_tasks).or(delete_task).with(cors);


warp::serve(routes)
.run(([127, 0, 0, 1], 3030))
.await;


Ok(())
}
