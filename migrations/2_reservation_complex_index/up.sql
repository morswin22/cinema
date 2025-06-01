ALTER TABLE reservation
ADD UNIQUE INDEX unique_user_schedule (user_id, schedule_id);
