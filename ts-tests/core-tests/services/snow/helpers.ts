function printLoading() {
	const P = ["\\", "|", "/", "-"];
	let x = 0;
	return setInterval(function () {
		process.stdout.write("\r" + P[x++]);
		x %= 4;
	}, 500);
}

export function sleep(sec: number) {
	const loading = printLoading();
	console.log(` Sleeping for ${sec} seconds`);
	return new Promise((resolve) => {
		setTimeout(() => {
			clearInterval(loading);
			resolve(true);
		}, sec * 1000);
	});
}
