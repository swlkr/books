app=books

echo "Shutting down server..."
killall $app
echo "Server shutdown"

cd $app
cp $app-new $app

echo "Starting server..."
./$app &> app.log &

echo "======================"
echo " DEPLOYMENT COMPLETED "
echo "======================"
