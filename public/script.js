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

	// Authenticate
	ws.send(
		JSON.stringify({
			Authenticate: {
				name: "Gavin",
				password: "123456",
			},
		})
	);
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

		// Hide login form
		document.getElementById("login").outerHTML = "<p>Logged in!</p>";

		// Show send message form
		document.getElementById("msg-form").style.display = "block";
	} else {
		console.error("Failed to authenticate:", message.Authenticate.error);
	}
}

function handleNewMessage(message) {
	console.log("New message:", message.NewMessage);

	message = message.NewMessage;

	const chat = document.getElementById("chat");
	makeMessageElem(message).insert();
}

function handleMessages(message) {
	console.log("Messages:", message.Messages);

	const messages = message.Messages;

	// Add to DOM
	const chat = document.getElementById("chat");

	for (const message of messages.reverse()) {
		makeMessageElem(message).insert();
	}
}

function makeMessageElem(message) {
	return new ChatMessage(message);
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
	const parent = document.getElementById("msg-input-parent").value;

	const msg = JSON.stringify({
		Message: {
			parent: parent.length <= 0 ? null : parent,
			content,
		},
	});

	console.log(msg);
	ws.send(msg);
});

class ChatMessage extends HTMLElement {
	static messages = [];

	/**
	 * @type {Object}
	 * @property {string} id
	 * @property {string} author
	 * @property {string} parent
	 * @property {string} content
	 */
	message;

	children = [];

	constructor(message) {
		super();

		this.message = message;

		// Create shadow DOM
		const shadow = this.attachShadow({ mode: "open" });

		const author = document.createElement("address");
		author.textContent = `[${message.author}]`;
		author.setAttribute("rel", "author");

		const space = document.createTextNode(" ");

		const content = document.createElement("span");
		content.textContent = message.content;

		const style = document.createElement("style");
		style.textContent = `
		address, span {
			margin-left: 0;
			display: inline-block;
		}

		address {
			color: gray;
			font-style: normal;
		}

		chat-message {
			margin-left: 1em;
			display: block;
			position: relative;
		}

		chat-message::before {
			content: "";
			display: block;
			width: 1px;
			height: 100%;
			background-color: gray;
			left: -1em;
			position: absolute;
		}
		`;

		shadow.appendChild(author);
		shadow.appendChild(space);
		shadow.appendChild(content);
		shadow.appendChild(style);

		ChatMessage.messages.push(this);
	}

	connectedCallback() {
		this.setAttribute("class", "message");
		this.setAttribute("id", `msg-${this.message.id}`);
		this.setAttribute("data-author", this.message.author);
		this.setAttribute("data-parent", this.message.parent);
		this.setAttribute("data-content", this.message.content);
		this.style.display = "block";
		this.style.position = "relative";
	}

	addChild(child) {
		// Add to DOM
		this.shadowRoot.appendChild(child);

		this.children.push(child);
	}

	insert(root = document.getElementById("chat")) {
		if (this.message.parent === "0") {
			root.appendChild(this);
			return;
		}

		const parent = ChatMessage.messages.filter(
			msg => msg.message.id === this.message.parent
		)[0];

		if (parent == null) {
			throw new Error(`Parent message not found: ${this.message.parent}`);
		}

		parent.addChild(this);
	}
}

customElements.define("chat-message", ChatMessage);
