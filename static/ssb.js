function whoami() {
	$.get("/api/whoami", function(response){
		$("#profile_username").text(response.username)
		$('#form_username').attr('value', response.username)
		$("#profile_description").html(micromarkdown.parse(response.description))
		$('#form_description').text(response.description)
		$("#profile_pubkey").text(response.pubkey)
	})
}

whoami()

window.addEventListener( "load", function () {
	function sendData() {
		const XHR = new XMLHttpRequest();

    	// Bind the FormData object and the form element
    	const FD = new FormData( form );

	    // Define what happens on successful data submission
    	XHR.addEventListener( "load", function(event) {
      		alert( event.target.responseText );
    	} );

    	// Define what happens in case of error
    	XHR.addEventListener( "error", function( event ) {
      		alert( 'Oops! Something went wrong.' );
    	} );

    	// Set up our request
    	XHR.open( "POST", "/api/update" );

    	// The data sent is what the user provided in the form
    	XHR.send( FD );
  	}

	// Access the form element...
	const form = document.getElementById( "profileForm" );

	// ...and take over its submit event.
	form.addEventListener( "submit", function ( event ) {
    	event.preventDefault();

    	sendData();
		whoami();
	} );
} );


