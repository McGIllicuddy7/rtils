async function main(){
    await put({x:"hello there", y:10});
}
main();

async function put(value){
    await fetch("",  {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body:JSON.stringify(value),
    });
}
