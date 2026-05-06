const button_class = "enable_only_if_cloudflare_turnstile_verifies";

const submit_button = document.querySelector(`.${button_class}`);
// disable on initial load
submit_button.disabled = true;

function enableSubmitButton() {
    submit_button.disabled = false;
}
function disableSubmitButton() {
    submit_button.disabled = true;
}