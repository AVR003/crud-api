use axum::{
    extract::{Path, State},
    routing::{delete,get,post,put},
    Router,
    Json,
};

use sqlx::PgPool;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
struct Student{
    id: i32,
    name: String,
    age: i32,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

async fn health() -> &'static str {
    "OK"
}

async fn hello() -> String {
    "Hello, from Rust API".to_string()
}

async fn get_students(
    State(state): State<AppState>,
) -> Json<Vec<Student>> {

    let students = sqlx::query_as::<_, Student>("SELECT * FROM students")
        .fetch_all(&state.db)
        .await
        .unwrap();

    Json(students)
}

async fn create_student(
    State(state): State<AppState>,
    Json(student): Json<Student>,
) -> Json<Student> {

    sqlx::query(
        "INSERT INTO students (id,name,age)
        VALUES ($1, $2, $3)"
    )
    .bind(student.id)
    .bind(&student.name)
    .bind(student.age)
    .execute(&state.db)
    .await
    .unwrap();

    Json(student)
}

async fn update_student(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(student): Json<Student>,
) -> Json<Student> {
    
    sqlx::query(
        "UPDATE students SET name = $1, age = $2 WHERE id = $3"
    )
    .bind(&student.name)
    .bind(student.age)
    .bind(id)
    .execute(&state.db)
    .await
    .unwrap();

    Json(student)
}

async fn delete_student(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> String{ 
    sqlx::query(
        "DELETE FROM students WHERE id = $1"
    )
    .bind(id)
    .execute(&state.db)
    .await
    .unwrap();
    format!("Student with id {} deleted", id)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url=
    std::env::var("DATABASE_URL").expect("DATABASE_URL not found");

    let pool=PgPool::connect(&database_url)
    .await.expect("Failed to connect to database");

    println!("Connected to database successfully");

    let state = AppState{
        db: pool,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/", get(hello))
        .route("/students", get(get_students))
        .route("/students", post(create_student))
        .route("/students/{id}", put(update_student))
        .route("/students/{id}", delete(delete_student))
        .with_state(state);
    
    let listener=
        tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
    .await
    .unwrap();
}


