const button_class = "enable_only_if_cloudflare_turnstile_verifies";

const cf_success_indicator_form_container_id = "cf_captcha_status_form";
const captcha_success_state_indicator_name = "cf_turnstile_verified";
const submit_button = document.querySelector(`.${button_class}`);


// disable on initial load
submit_button.disabled = true;
const cf_success_auto_indicator = getOrCreateCfStatusHiddenField();
 function script_cloudflare_turnstile__is_verified() {
    
    const answer= cf_success_auto_indicator.value === "true";
    return answer;
}
window.script_cloudflare_turnstile__is_verified = script_cloudflare_turnstile__is_verified;

function enableSubmitButton() {
    setCfValid(true);
    submit_button.disabled = false;
}
function disableSubmitButton() {
    setCfValid(false);
    submit_button.disabled = true;
}

function setCfValid(value) {
    cf_success_auto_indicator.value = value ? "true" : "false";
}

function getOrCreateCfStatusHiddenField() {
    let form = document.getElementById(cf_success_indicator_form_container_id);

    if (!form) {
        form = document.createElement("form");
        form.id = cf_success_indicator_form_container_id;
        form.style.display = "none";
        document.body.appendChild(form);
    }

    let input = document.getElementById(captcha_success_state_indicator_name);

    if (!input) {
        input = document.createElement("input");
        input.type = "hidden";
        input.id = captcha_success_state_indicator_name;
        input.name = captcha_success_state_indicator_name;
        input.value = "false";
        form.appendChild(input);
    }

    return input;
}