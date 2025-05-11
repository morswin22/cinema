// @generated automatically by Diesel CLI.

diesel::table! {                                                                                                                                        
    movies (id) {                                                                                                                                       
        id -> Integer,                                                                                                                                  
        #[max_length = 255]                                                                                                                             
        title -> Varchar,
        year -> Integer,                                                                                                                                
        #[max_length = 255]                                                                                                                             
        director -> Varchar,                                                                                                                            
    }                                                                                                                                                   
}

diesel::table! {                                                                                                                                        
    reservation (id) {                                                                                                                                  
        id -> Integer,
        user_id -> Integer,                                                                                                                             
        schedule_id -> Integer,                                                                                                                         
    }                                                                                                                                                   
}

diesel::table! {                                                                                                                                        
    rooms (id) {                                                                                                                                        
        id -> Integer,                                                                                                                                  
        capacity -> Integer,                                                                                                                            
        #[max_length = 50]                                                                                                                              
        label -> Varchar,
    }                                                                                                                                                   
}

diesel::table! {                                                                                                                                        
    schedule (id) {                                                                                                                                     
        id -> Integer,                                                                                                                                  
        movie_id -> Integer,                                                                                                                            
        room_id -> Integer,                                                                                                                             
        date -> Datetime,                                                                                                                               
    }
}

diesel::table! {                                                                                                                                        
    users (id) {                                                                                                                                        
        id -> Integer,                                                                                                                                  
        #[max_length = 255]                                                                                                                             
        email -> Varchar,                                                                                                                               
        #[max_length = 255]                                                                                                                             
        password -> Varchar,                                                                                                                            
    }
}

diesel::joinable!(reservation -> schedule (schedule_id));
diesel::joinable!(reservation -> users (user_id));
diesel::joinable!(schedule -> movies (movie_id));
diesel::joinable!(schedule -> rooms (room_id));

diesel::allow_tables_to_appear_in_same_query!(                                                                                                          
    movies,                                                                                                                                             
    reservation,
    rooms,                                                                                                                                              
    schedule,                                                                                                                                           
    users,                                                                                                                                              
);