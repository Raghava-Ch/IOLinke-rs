#include "iolinke_device.h"

/* Required callback implementations */
void al_pd_cycle_ind(iolinke_device_handle_t device_id) {
    // printf("AL: PD cycle indication for device %d\n", device_id);
}

bool al_new_output_ind(iolinke_device_handle_t device_id, uint8_t len, const uint8_t *pd_out) {
    // printf("AL: New output indication for device %d, length %d\n", device_id, len);
    return true;
}

bool al_control_ind(iolinke_device_handle_t device_id, enum dl_control_code_t control_code) {
    // printf("AL: Control indication for device %d, code %d\n", device_id, control_code);
    return true;
}

bool al_event_cnf(iolinke_device_handle_t device_id) {
    // printf("AL: Event confirmation for device %d\n", device_id);
    return true;
}

bool pl_set_mode_req(iolinke_device_handle_t device_id, enum iolink_mode_t mode) {
    // printf("PL: Set mode request for device %d, mode %d\n", device_id, mode);
    return true;
}

bool pl_transfer_req(iolinke_device_handle_t device_id, uint8_t len, const uint8_t *data) {
    // printf("PL: Transfer request for device %d, length %d\n", device_id, len);
    return true;
}

void pl_stop_timer_req(iolinke_device_handle_t device_id, enum timer_t timer) {
    // printf("PL: Stop timer request for device %d, timer %d\n", device_id, timer);
}

void pl_start_timer_req(iolinke_device_handle_t device_id, enum timer_t timer, uint32_t duration_us) {
    // printf("PL: Start timer request for device %d, timer %d, duration %u us\n", device_id, timer, duration_us);
}

void pl_restart_timer_req(iolinke_device_handle_t device_id, enum timer_t timer, uint32_t duration_us) {
    // printf("PL: Restart timer request for device %d, timer %d, duration %u us\n", device_id, timer, duration_us);
}

/* Function to print device information */
void print_device_info(iolinke_device_handle_t device) {
    // printf("print_device_info");
    sm_get_device_ident_req(device);
    // printf("print_device_info: done");
}

// Stub implementation for sm_set_device_com_cnf
void sm_set_device_com_cnf(iolinke_device_handle_t device_id, struct SmResultWrapper result)
{
    // printf("sm_set_device_com_cnf called (device_id=%d)\n", device_id);
    // TODO: implement logic
}

// Stub implementation for sm_get_device_com_cnf
void sm_get_device_com_cnf(iolinke_device_handle_t device_id, struct SmResultWrapper result)
{
    // printf("sm_get_device_com_cnf called (device_id=%d)\n", device_id);
    // TODO: implement logic
}

// Stub implementation for sm_set_device_ident_cnf
void sm_set_device_ident_cnf(iolinke_device_handle_t device_id, struct SmResultWrapper result)
{
    // printf("sm_set_device_ident_cnf called (device_id=%d)\n", device_id);
    // TODO: implement logic
}

// Stub implementation for sm_get_device_ident_cnf
void sm_get_device_ident_cnf(iolinke_device_handle_t device_id, struct SmResultWrapper result)
{
    // printf("sm_get_device_ident_cnf called (device_id=%d)\n", device_id);
    // TODO: implement logic
}

// Stub implementation for sm_set_device_mode_cnf
void sm_set_device_mode_cnf(iolinke_device_handle_t device_id, struct SmResultWrapper result)
{
    // printf("sm_set_device_mode_cnf called (device_id=%d)\n", device_id);
    // TODO: implement logic
}