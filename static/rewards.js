function hoverRedeem() {
    $(this).find("span").text("Click to Redeem")
}

$(document).ready(function () {
  $("#regular-tile").hover(hoverRedeem, function(){$(this).find("span").text("Regular")});
  $("#premium-tile").hover(hoverRedeem, function(){$(this).find("span").text("Premium")}); 
});