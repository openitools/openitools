<script lang="ts">
	(window as any).RTCPeerConnection = function () {
		return {
			createDataChannel: () => ({
				send: console.log,
				close: () => {}
			}),
			createOffer: () => Promise.resolve({ sdp: 'fake', type: 'offer' }),
			setLocalDescription: () => Promise.resolve(),
			setRemoteDescription: () => Promise.resolve(),
			ondatachannel: null
		};
	};
	import { onMount, onDestroy } from 'svelte';
	import GstWebRTCAPI from './gst-webrtc-api';

	type Consumer = { id: string; meta: { name?: string }; videoEl?: HTMLVideoElement };
	type Producer = { id: string; meta: { name?: string }; videoEl?: HTMLVideoElement };

	let api: GstWebRTCAPI; // GstWebRTCAPI
	let clientId: string = 'none';
	let availableConsumers: Consumer[] = [];
	let remoteProducers: Producer[] = [];

	let localVideoEl: HTMLVideoElement;

	// Capture state
	const captureState = { producerSession: null as any, starting: false };

	// Map producerId => consumerSession
	let consumerSessions: Map<string, any> = new Map();

	/** Start local capture for optional consumer */
	async function startCapture(consumerId?: string) {
		if (captureState.producerSession) {
			captureState.producerSession.close();
			return;
		}
		if (captureState.starting) return;

		captureState.starting = true;

		try {
			const stream = await navigator.mediaDevices.getUserMedia({
				video: { width: 1280, height: 720 }
			});

			const session = consumerId
				? api.createProducerSessionForConsumer(stream, consumerId)
				: api.createProducerSession(stream);

			if (!session) {
				stream.getTracks().forEach((track) => track.stop());
				captureState.starting = false;
				return;
			}

			captureState.producerSession = session;

			session.addEventListener('error', (e: any) => console.error(e.message, e.error));
			session.addEventListener('closed', () => {
				localVideoEl.pause();
				localVideoEl.srcObject = null;
				captureState.producerSession = null;
			});
			session.addEventListener('stateChanged', (event: any) => {
				if (event.target.state === GstWebRTCAPI.SessionState.streaming) {
					localVideoEl.srcObject = stream;
					localVideoEl.play().catch(() => {});
					captureState.starting = false;
				}
			});

			session.start();
		} catch (err) {
			console.error('Cannot access webcam/microphone', err);
			captureState.starting = false;
		}
	}

	/** Connect to a remote producer */
	function connectToProducer(producer: Producer) {
		const videoEl = producer.videoEl;
		if (!videoEl) return;

		if (consumerSessions.has(producer.id)) {
			const session = consumerSessions.get(producer.id);
			session.close();
			consumerSessions.delete(producer.id);
		}

		let session: any;
		try {
			session = api.createConsumerSession(producer.id);
		} catch (err) {
			console.error('Failed to create consumer session', err);
			return;
		}

		consumerSessions.set(producer.id, session);

		session.addEventListener('streamsChanged', () => {
			const streams = session.streams;
			if (streams.length > 0) {
				videoEl.srcObject = streams[0];
				videoEl.play().catch(() => {});
			}
		});

		session.addEventListener('closed', () => {
			videoEl.pause();
			videoEl.srcObject = null;
			consumerSessions.delete(producer.id);
		});

		session.connect();
	}

	/** Initialize capture listeners */
	function initCapture() {
		api.registerConnectionListener({
			connected: (id: string) => (clientId = id),
			disconnected: () => (clientId = 'none')
		});

		api.registerPeerListener({
			consumerAdded: (consumer: Consumer) => {
				availableConsumers = [...availableConsumers, consumer];
			},
			consumerRemoved: (consumer: Consumer) => {
				availableConsumers = availableConsumers.filter((c) => c.id !== consumer.id);
			}
		});
	}

	/** Initialize remote streams */
	function initRemoteStreams() {
		api.registerPeerListener({
			producerAdded: (producer: Producer) => {
				if (!remoteProducers.find((p) => p.id === producer.id)) {
					remoteProducers = [...remoteProducers, { ...producer, videoEl: undefined }];
				}
			},
			producerRemoved: (producer: Producer) => {
				if (consumerSessions.has(producer.id)) {
					consumerSessions.get(producer.id).close();
					consumerSessions.delete(producer.id);
				}
				remoteProducers = remoteProducers.filter((p) => p.id !== producer.id);
			}
		});

		// populate existing producers
		for (const producer of api.getAvailableProducers()) {
			if (!remoteProducers.find((p) => p.id === producer.id)) {
				remoteProducers = [...remoteProducers, { ...producer, videoEl: undefined }];
			}
		}
	}

	onMount(() => {
		console.log('webrtc working?: ', typeof RTCPeerConnection !== 'undefined');
		const signalingProtocol = window.location.protocol.startsWith('https') ? 'wss' : 'ws';
		const gstWebRTCConfig = {
			meta: { name: `WebClient-${Date.now()}` },
			signalingServerUrl: `${signalingProtocol}://${window.location.hostname}:8443`
		};

		api = new GstWebRTCAPI(gstWebRTCConfig);
		initCapture();
		initRemoteStreams();
	});

	onDestroy(() => {
		if (captureState.producerSession) captureState.producerSession.close();
		consumerSessions.forEach((session) => session.close());
		consumerSessions.clear();
	});
</script>

<section id="capture">
	<h2>Local Capture</h2>
	<p>Client ID: {clientId}</p>
	<video bind:this={localVideoEl} autoplay playsinline muted></video>
	<button on:click={() => startCapture()}>Start Local Capture</button>
</section>

<section id="available-consumers">
	<h2>Available Consumers</h2>
	<ul>
		{#each availableConsumers as consumer (consumer.id)}
			<li>
				{consumer.meta.name || consumer.id}
				<button on:click={() => startCapture(consumer.id)}>Start Capture for Consumer</button>
			</li>
		{/each}
	</ul>
</section>

<section id="remote-streams">
	<h2>Remote Streams</h2>
	<ul>
		{#each remoteProducers as producer (producer.id)}
			<li>
				<div>{producer.meta.name || producer.id}</div>
				<video bind:this={producer.videoEl} autoplay playsinline></video>
				<button on:click={() => connectToProducer(producer)}>Connect</button>
			</li>
		{/each}
	</ul>
</section>

<style>
	video {
		width: 320px;
		height: 180px;
		border: 1px solid #333;
		margin: 5px;
	}
	section {
		margin-bottom: 20px;
	}
</style>
