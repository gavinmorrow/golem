// Connect to the websocket endpoint
const HOST = "localhost:7878";
const ws = new WebSocket(`ws://${HOST}/api/ws/`);

ws.addEventListener("message", event => {
	const data = JSON.parse(event.data);
	if (data != null) handleMessage(JSON.parse(event.data));
	else console.error("Invalid message received:", event.data);
});

ws.addEventListener("open", event => {
	console.log("Connected to the websocket endpoint!");

	// Load messages
	getMessages(ws);
});

ws.addEventListener("error", event => {
	console.log("Error", event);
});

ws.addEventListener("close", event => {
	console.log("Disconnected from the websocket endpoint!");
});

function getMessages(ws) {
	ws.send(
		JSON.stringify({
			LoadAllMessages: null,
		})
	);
}

function handleMessage(message) {
	console.log("Got message:", message);
	switch (Object.keys(message)[0]) {
		case "Authenticate":
			handleAuthenticate(message);
			break;
		case "NewMessage":
			handleNewMessage(message);
			break;
		case "Messages":
			handleMessages(message);
			break;
		case "Error":
			handleError(message);
			break;
		default:
			console.error("Unknown message type:", message);
	}
}

function handleAuthenticate(message) {
	if (message.Authenticate.success) {
		console.log("Successfully authenticated!");
	} else {
		console.error("Failed to authenticate:", message.Authenticate.error);
	}
}

function handleNewMessage(message) {
	console.log("New message:", message.NewMessage);

	message = message.NewMessage;

	const chat = document.getElementById("chat");

	const msg = document.createElement("div");
	msg.className = "message";
	msg.innerHTML = `<span class="message-author">${message.author}</span>: ${message.content}`;
	chat.appendChild(msg);
}

function handleMessages(message) {
	console.log("Messages:", message.Messages);

	const messages = message.Messages;

	// Add to DOM
	const chat = document.getElementById("chat");

	for (const message of messages) {
		const msg = document.createElement("div");
		msg.className = "message";
		msg.innerHTML = `<span class="message-author">${message.author}</span>: ${message.content}`;
		chat.prepend(msg);
	}
}

function handleError(message) {
	console.error("Error:", message.Error);
}

document.getElementById("login").addEventListener("submit", event => {
	event.preventDefault();
	const name = document.getElementById("name").value;
	const password = document.getElementById("password").value;

	ws.send(
		JSON.stringify({
			Authenticate: {
				name,
				password,
			},
		})
	);
});

document.getElementById("msg-form").addEventListener("submit", event => {
	event.preventDefault();
	const content = document.getElementById("msg-input").value;

	const msg = JSON.stringify({
		Message: {
			parent: null,
			content,
		},
	});

	console.log(msg);
	ws.send(msg);
});
