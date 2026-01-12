function main(){
    fetch("",  {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body:JSON.stringify("hello world"),
    });
}
main();