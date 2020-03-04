var menu_btn = document.getElementById("menu-btn");
var menu_a = document.querySelectorAll(".menu-a");

checkbox = () => {
    if(menu_btn.checked === false) {
        menu_btn.checked = true; 
    } else {
        menu_btn.checked = false;
    }
}

for (var i = 0; i < menu_a.length; i++) {
    menu_a[i].addEventListener('click', checkbox);
}