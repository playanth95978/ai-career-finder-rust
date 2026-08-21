import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { IJobApplication } from '../job-application.model';
import { sampleWithFullData, sampleWithNewData, sampleWithPartialData, sampleWithRequiredData } from '../job-application.test-samples';

import { JobApplicationService, RestJobApplication } from './job-application.service';

const requireRestSample: RestJobApplication = {
  ...sampleWithRequiredData,
  createdAt: sampleWithRequiredData.createdAt?.toJSON(),
  updatedAt: sampleWithRequiredData.updatedAt?.toJSON(),
  appliedAt: sampleWithRequiredData.appliedAt?.toJSON(),
};

describe('JobApplication Service', () => {
  let service: JobApplicationService;
  let httpMock: HttpTestingController;
  let expectedResult: IJobApplication | IJobApplication[] | boolean | null;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClientTesting()],
    });
    expectedResult = null;
    service = TestBed.inject(JobApplicationService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  describe('Service methods', () => {
    it('should find an element', () => {
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.find(123).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should create a JobApplication', () => {
      const jobApplication = { ...sampleWithNewData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.create(jobApplication).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'POST' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should update a JobApplication', () => {
      const jobApplication = { ...sampleWithRequiredData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.update(jobApplication).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PUT' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should partial update a JobApplication', () => {
      const patchObject = { ...sampleWithPartialData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.partialUpdate(patchObject).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PATCH' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should return a list of JobApplication', () => {
      const returnedFromService = { ...requireRestSample };

      const expected = { ...sampleWithRequiredData };

      service.query().subscribe(resp => (expectedResult = resp.body));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush([returnedFromService]);
      httpMock.verify();
      expect(expectedResult).toMatchObject([expected]);
    });

    it('should delete a JobApplication', () => {
      service.delete(123).subscribe();

      const requests = httpMock.match({ method: 'DELETE' });
      expect(requests).toHaveLength(1);
    });

    describe('addJobApplicationToCollectionIfMissing', () => {
      it('should add a JobApplication to an empty array', () => {
        const jobApplication: IJobApplication = sampleWithRequiredData;
        expectedResult = service.addJobApplicationToCollectionIfMissing([], jobApplication);
        expect(expectedResult).toEqual([jobApplication]);
      });

      it('should not add a JobApplication to an array that contains it', () => {
        const jobApplication: IJobApplication = sampleWithRequiredData;
        const jobApplicationCollection: IJobApplication[] = [
          {
            ...jobApplication,
          },
          sampleWithPartialData,
        ];
        expectedResult = service.addJobApplicationToCollectionIfMissing(jobApplicationCollection, jobApplication);
        expect(expectedResult).toHaveLength(2);
      });

      it("should add a JobApplication to an array that doesn't contain it", () => {
        const jobApplication: IJobApplication = sampleWithRequiredData;
        const jobApplicationCollection: IJobApplication[] = [sampleWithPartialData];
        expectedResult = service.addJobApplicationToCollectionIfMissing(jobApplicationCollection, jobApplication);
        expect(expectedResult).toHaveLength(2);
        expect(expectedResult).toContain(jobApplication);
      });

      it('should add only unique JobApplication to an array', () => {
        const jobApplicationArray: IJobApplication[] = [sampleWithRequiredData, sampleWithPartialData, sampleWithFullData];
        const jobApplicationCollection: IJobApplication[] = [sampleWithRequiredData];
        expectedResult = service.addJobApplicationToCollectionIfMissing(jobApplicationCollection, ...jobApplicationArray);
        expect(expectedResult).toHaveLength(3);
      });

      it('should accept varargs', () => {
        const jobApplication: IJobApplication = sampleWithRequiredData;
        const jobApplication2: IJobApplication = sampleWithPartialData;
        expectedResult = service.addJobApplicationToCollectionIfMissing([], jobApplication, jobApplication2);
        expect(expectedResult).toEqual([jobApplication, jobApplication2]);
      });

      it('should accept null and undefined values', () => {
        const jobApplication: IJobApplication = sampleWithRequiredData;
        expectedResult = service.addJobApplicationToCollectionIfMissing([], null, jobApplication, undefined);
        expect(expectedResult).toEqual([jobApplication]);
      });

      it('should return initial array if no JobApplication is added', () => {
        const jobApplicationCollection: IJobApplication[] = [sampleWithRequiredData];
        expectedResult = service.addJobApplicationToCollectionIfMissing(jobApplicationCollection, undefined, null);
        expect(expectedResult).toEqual(jobApplicationCollection);
      });
    });

    describe('compareJobApplication', () => {
      it('should return true if both entities are null', () => {
        const entity1 = null;
        const entity2 = null;

        const compareResult = service.compareJobApplication(entity1, entity2);

        expect(compareResult).toEqual(true);
      });

      it('should return false if one entity is null', () => {
        const entity1 = { id: 20361 };
        const entity2 = null;

        const compareResult1 = service.compareJobApplication(entity1, entity2);
        const compareResult2 = service.compareJobApplication(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey differs', () => {
        const entity1 = { id: 20361 };
        const entity2 = { id: 562 };

        const compareResult1 = service.compareJobApplication(entity1, entity2);
        const compareResult2 = service.compareJobApplication(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey matches', () => {
        const entity1 = { id: 20361 };
        const entity2 = { id: 20361 };

        const compareResult1 = service.compareJobApplication(entity1, entity2);
        const compareResult2 = service.compareJobApplication(entity2, entity1);

        expect(compareResult1).toEqual(true);
        expect(compareResult2).toEqual(true);
      });
    });
  });

  afterEach(() => {
    httpMock.verify();
  });
});
