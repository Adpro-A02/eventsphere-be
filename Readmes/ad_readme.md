# Manajemen Iklan

The advertisement management service is responsible for managing advertisements in the system. It provides functionality to create, update, delete, and retrieve advertisements. The service interacts with the database to store and manage advertisement data.

for this service. I am using these patterns:
1. **Repository Pattern**: This pattern is used to abstract the data access layer and provide a clean API for the service to interact with the database. It allows for easy switching of the underlying data storage mechanism without affecting the service logic.
2. **Service Pattern**: This pattern is used to encapsulate the business logic of the advertisement management service. It provides a clear separation of concerns and allows for easy testing and maintenance of the service logic.
3. **Dependency Injection**: This pattern is used to manage the dependencies of the service and its components. It allows for easy swapping of implementations and promotes loose coupling between components.
4. **DTO (Data Transfer Object) Pattern**: This pattern is used to transfer data between the service and its clients. It allows for easy serialization and deserialization of data and provides a clear contract for the data being exchanged.
5. **Query Object Pattern**: Handles search parameters and pagination elegantly, simplifying complex query construction.

These patterns align with core design principles like Single Responsibility, Interface Segregation, and Dependency Inversion. Together, they create a clean separation of concerns while maintaining flexibility for future changes, resulting in a maintainable system that can easily adapt to evolving requirements.