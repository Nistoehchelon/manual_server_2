const submit_button_id = "submit_button";

const submit_button = document.getElementById(submit_button_id);
// disable on initial load
submit_button.disabled = true;

function enableSubmitButton() {
    submit_button.disabled = false;
}
function disableSubmitButton() {
    submit_button.disabled = true;
}