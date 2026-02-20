const cookie_name = "alert_cookie";
const cookie_found = !(document.cookie.indexOf(cookie_name) === -1);
if (!cookie_found) {
    alert("Welcome!");
    const one_day = 86400;
    document.cookie = `${cookie_name}=tehee; path=/; secure; max-age=${one_day}`;
}