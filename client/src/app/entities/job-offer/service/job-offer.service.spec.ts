import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { IJobOffer } from '../job-offer.model';
import { sampleWithFullData, sampleWithNewData, sampleWithPartialData, sampleWithRequiredData } from '../job-offer.test-samples';

import { JobOfferService, RestJobOffer } from './job-offer.service';

const requireRestSample: RestJobOffer = {
  ...sampleWithRequiredData,
  publishedAt: sampleWithRequiredData.publishedAt?.toJSON(),
  createdAt: sampleWithRequiredData.createdAt?.toJSON(),
  indexedAt: sampleWithRequiredData.indexedAt?.toJSON(),
  updatedAt: sampleWithRequiredData.updatedAt?.toJSON(),
  expiresAt: sampleWithRequiredData.expiresAt?.toJSON(),
  lastCheckedAt: sampleWithRequiredData.lastCheckedAt?.toJSON(),
};

describe('JobOffer Service', () => {
  let service: JobOfferService;
  let httpMock: HttpTestingController;
  let expectedResult: IJobOffer | IJobOffer[] | boolean | null;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClientTesting()],
    });
    expectedResult = null;
    service = TestBed.inject(JobOfferService);
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

    it('should create a JobOffer', () => {
      const jobOffer = { ...sampleWithNewData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.create(jobOffer).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'POST' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should update a JobOffer', () => {
      const jobOffer = { ...sampleWithRequiredData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.update(jobOffer).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PUT' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should partial update a JobOffer', () => {
      const patchObject = { ...sampleWithPartialData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.partialUpdate(patchObject).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PATCH' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should return a list of JobOffer', () => {
      const returnedFromService = { ...requireRestSample };

      const expected = { ...sampleWithRequiredData };

      service.query().subscribe(resp => (expectedResult = resp.body));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush([returnedFromService]);
      httpMock.verify();
      expect(expectedResult).toMatchObject([expected]);
    });

    it('should delete a JobOffer', () => {
      service.delete(123).subscribe();

      const requests = httpMock.match({ method: 'DELETE' });
      expect(requests).toHaveLength(1);
    });

    describe('addJobOfferToCollectionIfMissing', () => {
      it('should add a JobOffer to an empty array', () => {
        const jobOffer: IJobOffer = sampleWithRequiredData;
        expectedResult = service.addJobOfferToCollectionIfMissing([], jobOffer);
        expect(expectedResult).toEqual([jobOffer]);
      });

      it('should not add a JobOffer to an array that contains it', () => {
        const jobOffer: IJobOffer = sampleWithRequiredData;
        const jobOfferCollection: IJobOffer[] = [
          {
            ...jobOffer,
          },
          sampleWithPartialData,
        ];
        expectedResult = service.addJobOfferToCollectionIfMissing(jobOfferCollection, jobOffer);
        expect(expectedResult).toHaveLength(2);
      });

      it("should add a JobOffer to an array that doesn't contain it", () => {
        const jobOffer: IJobOffer = sampleWithRequiredData;
        const jobOfferCollection: IJobOffer[] = [sampleWithPartialData];
        expectedResult = service.addJobOfferToCollectionIfMissing(jobOfferCollection, jobOffer);
        expect(expectedResult).toHaveLength(2);
        expect(expectedResult).toContain(jobOffer);
      });

      it('should add only unique JobOffer to an array', () => {
        const jobOfferArray: IJobOffer[] = [sampleWithRequiredData, sampleWithPartialData, sampleWithFullData];
        const jobOfferCollection: IJobOffer[] = [sampleWithRequiredData];
        expectedResult = service.addJobOfferToCollectionIfMissing(jobOfferCollection, ...jobOfferArray);
        expect(expectedResult).toHaveLength(3);
      });

      it('should accept varargs', () => {
        const jobOffer: IJobOffer = sampleWithRequiredData;
        const jobOffer2: IJobOffer = sampleWithPartialData;
        expectedResult = service.addJobOfferToCollectionIfMissing([], jobOffer, jobOffer2);
        expect(expectedResult).toEqual([jobOffer, jobOffer2]);
      });

      it('should accept null and undefined values', () => {
        const jobOffer: IJobOffer = sampleWithRequiredData;
        expectedResult = service.addJobOfferToCollectionIfMissing([], null, jobOffer, undefined);
        expect(expectedResult).toEqual([jobOffer]);
      });

      it('should return initial array if no JobOffer is added', () => {
        const jobOfferCollection: IJobOffer[] = [sampleWithRequiredData];
        expectedResult = service.addJobOfferToCollectionIfMissing(jobOfferCollection, undefined, null);
        expect(expectedResult).toEqual(jobOfferCollection);
      });
    });

    describe('compareJobOffer', () => {
      it('should return true if both entities are null', () => {
        const entity1 = null;
        const entity2 = null;

        const compareResult = service.compareJobOffer(entity1, entity2);

        expect(compareResult).toEqual(true);
      });

      it('should return false if one entity is null', () => {
        const entity1 = { id: 9246 };
        const entity2 = null;

        const compareResult1 = service.compareJobOffer(entity1, entity2);
        const compareResult2 = service.compareJobOffer(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey differs', () => {
        const entity1 = { id: 9246 };
        const entity2 = { id: 5985 };

        const compareResult1 = service.compareJobOffer(entity1, entity2);
        const compareResult2 = service.compareJobOffer(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey matches', () => {
        const entity1 = { id: 9246 };
        const entity2 = { id: 9246 };

        const compareResult1 = service.compareJobOffer(entity1, entity2);
        const compareResult2 = service.compareJobOffer(entity2, entity1);

        expect(compareResult1).toEqual(true);
        expect(compareResult2).toEqual(true);
      });
    });
  });

  afterEach(() => {
    httpMock.verify();
  });
});
