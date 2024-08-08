
$(window).on("click", function(e) {
  let el = $(e.target);
  if( el.parents('.dropdown-menu').length > 0 || el.hasClass(".dropdown-menu")) {
  } 
  else if( el.parents('.dropdown-trigger').length > 0 || el.hasClass(".dropdown-trigger") ) {
    el.parents(".dropdown").toggleClass("is-active");
  }
  else {
    $(".dropdown.is-active").removeClass("is-active");
  }
});